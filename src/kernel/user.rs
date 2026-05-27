/// User account, login, and session management.
///
/// The current kernel has no hardware entropy source or full crypto support, so
/// password storage uses a salted deterministic hash as a replaceable kernel
/// primitive. Keep the API boundary here when adding a stronger hasher later.

use spin::Mutex;

use crate::driver::storage::{self, BLOCK_SIZE};

pub const MAX_USERS: usize = 64;
pub const USERNAME_MAX: usize = 32;
pub const PASSWORD_MIN: usize = 6;
pub const MAX_LOGIN_FAILURES: u8 = 5;

const USER_DB_BLOCK: u64 = 2;
const USER_DB_MAGIC: &[u8; 8] = b"VZXUSER1";
const USER_DB_VERSION: u8 = 1;
const USER_RECORD_SIZE: usize = 63;
const DEFAULT_ADMIN_NAME: &str = "admin";
const DEFAULT_ADMIN_PASSWORD: &str = "admin123";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    Standard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserStatus {
    Active,
    Disabled,
    Locked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserError {
    InvalidUsername,
    InvalidPassword,
    UserExists,
    UserNotFound,
    UserTableFull,
    PermissionDenied,
    StorageUnavailable,
    CorruptDatabase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginError {
    InvalidCredentials,
    UserDisabled,
    UserLocked,
}

#[derive(Clone, Copy)]
pub struct User {
    pub id: u32,
    pub username: [u8; USERNAME_MAX],
    pub username_len: usize,
    password_hash: u64,
    salt: u64,
    pub role: UserRole,
    pub status: UserStatus,
    pub failed_logins: u8,
}

impl User {
    pub fn username_matches(&self, username: &str) -> bool {
        username.as_bytes() == &self.username[..self.username_len]
    }

    pub fn username_bytes(&self) -> &[u8] {
        &self.username[..self.username_len]
    }
}

#[derive(Clone, Copy)]
pub struct Session {
    pub user_id: u32,
    pub role: UserRole,
    pub started_at_ticks: u64,
}

pub struct UserManager {
    users: [Option<User>; MAX_USERS],
    user_count: usize,
    next_id: u32,
    current_session: Option<Session>,
}

impl UserManager {
    pub const fn empty() -> Self {
        UserManager {
            users: [None; MAX_USERS],
            user_count: 0,
            next_id: 1,
            current_session: None,
        }
    }

    pub fn reset_with_default_admin(&mut self) {
        self.users = [None; MAX_USERS];
        self.user_count = 0;
        self.next_id = 1;
        self.current_session = None;

        let _ = self.create_user(DEFAULT_ADMIN_NAME, DEFAULT_ADMIN_PASSWORD, UserRole::Admin);
    }

    pub fn create_user(
        &mut self,
        username: &str,
        password: &str,
        role: UserRole,
    ) -> Result<u32, UserError> {
        validate_username(username)?;
        validate_password(password)?;

        if self.find_index(username).is_some() {
            return Err(UserError::UserExists);
        }

        let slot = self
            .users
            .iter_mut()
            .find(|slot| slot.is_none())
            .ok_or(UserError::UserTableFull)?;

        let id = self.next_id;
        self.next_id += 1;

        let salt = make_salt(id, username);
        *slot = Some(User {
            id,
            username: copy_username(username),
            username_len: username.len(),
            password_hash: hash_password(password, salt),
            salt,
            role,
            status: UserStatus::Active,
            failed_logins: 0,
        });
        self.user_count += 1;

        Ok(id)
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<Session, LoginError> {
        let index = self.find_index(username).ok_or(LoginError::InvalidCredentials)?;
        let mut user = self.users[index].ok_or(LoginError::InvalidCredentials)?;

        match user.status {
            UserStatus::Disabled => return Err(LoginError::UserDisabled),
            UserStatus::Locked => return Err(LoginError::UserLocked),
            UserStatus::Active => {}
        }

        if user.password_hash != hash_password(password, user.salt) {
            user.failed_logins = user.failed_logins.saturating_add(1);
            if user.failed_logins >= MAX_LOGIN_FAILURES {
                user.status = UserStatus::Locked;
            }
            self.users[index] = Some(user);
            return Err(LoginError::InvalidCredentials);
        }

        user.failed_logins = 0;
        self.users[index] = Some(user);

        let session = Session {
            user_id: user.id,
            role: user.role,
            started_at_ticks: 0,
        };
        self.current_session = Some(session);

        Ok(session)
    }

    pub fn logout(&mut self) {
        self.current_session = None;
    }

    pub fn current_session(&self) -> Option<Session> {
        self.current_session
    }

    pub fn current_user(&self) -> Option<User> {
        let session = self.current_session?;
        self.find_by_id(session.user_id)
    }

    pub fn user_count(&self) -> usize {
        self.user_count
    }

    pub fn list_users(&self) -> [Option<User>; MAX_USERS] {
        self.users
    }

    pub fn set_role(&mut self, user_id: u32, role: UserRole) -> Result<(), UserError> {
        self.require_admin()?;
        let index = self.find_index_by_id(user_id).ok_or(UserError::UserNotFound)?;
        if let Some(mut user) = self.users[index] {
            user.role = role;
            self.users[index] = Some(user);
            Ok(())
        } else {
            Err(UserError::UserNotFound)
        }
    }

    pub fn set_status(&mut self, user_id: u32, status: UserStatus) -> Result<(), UserError> {
        self.require_admin()?;
        let index = self.find_index_by_id(user_id).ok_or(UserError::UserNotFound)?;
        if let Some(mut user) = self.users[index] {
            user.status = status;
            if status == UserStatus::Active {
                user.failed_logins = 0;
            }
            self.users[index] = Some(user);
            Ok(())
        } else {
            Err(UserError::UserNotFound)
        }
    }

    pub fn delete_user(&mut self, user_id: u32) -> Result<(), UserError> {
        self.require_admin()?;
        let index = self.find_index_by_id(user_id).ok_or(UserError::UserNotFound)?;

        if self.current_session.map(|session| session.user_id) == Some(user_id) {
            return Err(UserError::PermissionDenied);
        }

        self.users[index] = None;
        self.user_count -= 1;
        Ok(())
    }

    pub fn change_password(
        &mut self,
        user_id: u32,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), UserError> {
        validate_password(new_password)?;

        let index = self.find_index_by_id(user_id).ok_or(UserError::UserNotFound)?;
        let mut user = self.users[index].ok_or(UserError::UserNotFound)?;

        if user.password_hash != hash_password(old_password, user.salt) {
            return Err(UserError::PermissionDenied);
        }

        user.salt = make_salt(user.id, username_as_str(&user));
        user.password_hash = hash_password(new_password, user.salt);
        self.users[index] = Some(user);
        Ok(())
    }

    pub fn reset_password_as_admin(
        &mut self,
        user_id: u32,
        new_password: &str,
    ) -> Result<(), UserError> {
        self.require_admin()?;
        validate_password(new_password)?;

        let index = self.find_index_by_id(user_id).ok_or(UserError::UserNotFound)?;
        let mut user = self.users[index].ok_or(UserError::UserNotFound)?;

        user.salt = make_salt(user.id, username_as_str(&user));
        user.password_hash = hash_password(new_password, user.salt);
        user.failed_logins = 0;
        if user.status == UserStatus::Locked {
            user.status = UserStatus::Active;
        }
        self.users[index] = Some(user);
        Ok(())
    }

    pub fn save_to_storage(&self) -> Result<(), UserError> {
        let mut buffer = [0u8; BLOCK_SIZE];
        serialize_users(self, &mut buffer)?;
        storage::write_block(USER_DB_BLOCK, &buffer).map_err(|_| UserError::StorageUnavailable)
    }

    pub fn load_from_storage(&mut self) -> Result<(), UserError> {
        let mut buffer = [0u8; BLOCK_SIZE];
        storage::read_block(USER_DB_BLOCK, &mut buffer)
            .map_err(|_| UserError::StorageUnavailable)?;
        deserialize_users(self, &buffer)
    }

    fn require_admin(&self) -> Result<(), UserError> {
        let session = self.current_session.ok_or(UserError::PermissionDenied)?;
        match self.find_by_id(session.user_id) {
            Some(user) if user.role == UserRole::Admin && user.status == UserStatus::Active => {
                Ok(())
            }
            _ => Err(UserError::PermissionDenied),
        }
    }

    fn find_index(&self, username: &str) -> Option<usize> {
        self.users.iter().position(|slot| {
            if let Some(user) = slot {
                return user.username_matches(username);
            }
            false
        })
    }

    fn find_index_by_id(&self, user_id: u32) -> Option<usize> {
        self.users.iter().position(|slot| {
            if let Some(user) = slot {
                return user.id == user_id;
            }
            false
        })
    }

    fn find_by_id(&self, user_id: u32) -> Option<User> {
        self.users.iter().find_map(|slot| {
            if let Some(user) = slot {
                if user.id == user_id {
                    return Some(*user);
                }
            }
            None
        })
    }
}

static USER_MANAGER: Mutex<UserManager> = Mutex::new(UserManager::empty());

pub fn init() {
    let mut manager = USER_MANAGER.lock();
    manager.reset_with_default_admin();
}

pub fn load_or_create_default() -> Result<(), UserError> {
    let mut manager = USER_MANAGER.lock();
    match manager.load_from_storage() {
        Ok(()) => Ok(()),
        Err(UserError::CorruptDatabase) => {
            manager.reset_with_default_admin();
            manager.save_to_storage()
        }
        Err(err) => Err(err),
    }
}

pub fn register_user(username: &str, password: &str) -> Result<u32, UserError> {
    let mut manager = USER_MANAGER.lock();
    let id = manager.create_user(username, password, UserRole::Standard)?;
    let _ = manager.save_to_storage();
    Ok(id)
}

pub fn create_user_as_admin(
    username: &str,
    password: &str,
    role: UserRole,
) -> Result<u32, UserError> {
    let mut manager = USER_MANAGER.lock();
    manager.require_admin()?;
    let id = manager.create_user(username, password, role)?;
    let _ = manager.save_to_storage();
    Ok(id)
}

pub fn login(username: &str, password: &str) -> Result<Session, LoginError> {
    let mut manager = USER_MANAGER.lock();
    let result = manager.login(username, password);
    let _ = manager.save_to_storage();
    result
}

pub fn logout() {
    let mut manager = USER_MANAGER.lock();
    manager.logout();
}

pub fn current_session() -> Option<Session> {
    USER_MANAGER.lock().current_session()
}

pub fn current_user() -> Option<User> {
    USER_MANAGER.lock().current_user()
}

pub fn list_users() -> [Option<User>; MAX_USERS] {
    USER_MANAGER.lock().list_users()
}

pub fn user_count() -> usize {
    USER_MANAGER.lock().user_count()
}

pub fn set_user_role(user_id: u32, role: UserRole) -> Result<(), UserError> {
    let mut manager = USER_MANAGER.lock();
    manager.set_role(user_id, role)?;
    let _ = manager.save_to_storage();
    Ok(())
}

pub fn set_user_status(user_id: u32, status: UserStatus) -> Result<(), UserError> {
    let mut manager = USER_MANAGER.lock();
    manager.set_status(user_id, status)?;
    let _ = manager.save_to_storage();
    Ok(())
}

pub fn delete_user(user_id: u32) -> Result<(), UserError> {
    let mut manager = USER_MANAGER.lock();
    manager.delete_user(user_id)?;
    let _ = manager.save_to_storage();
    Ok(())
}

pub fn change_password(
    user_id: u32,
    old_password: &str,
    new_password: &str,
) -> Result<(), UserError> {
    let mut manager = USER_MANAGER.lock();
    manager.change_password(user_id, old_password, new_password)?;
    let _ = manager.save_to_storage();
    Ok(())
}

pub fn reset_password_as_admin(user_id: u32, new_password: &str) -> Result<(), UserError> {
    let mut manager = USER_MANAGER.lock();
    manager.reset_password_as_admin(user_id, new_password)?;
    let _ = manager.save_to_storage();
    Ok(())
}

fn validate_username(username: &str) -> Result<(), UserError> {
    let len = username.len();
    if len < 3 || len > USERNAME_MAX {
        return Err(UserError::InvalidUsername);
    }

    for byte in username.bytes() {
        let valid = byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-';
        if !valid {
            return Err(UserError::InvalidUsername);
        }
    }

    Ok(())
}

fn validate_password(password: &str) -> Result<(), UserError> {
    if password.len() < PASSWORD_MIN {
        return Err(UserError::InvalidPassword);
    }
    Ok(())
}

fn copy_username(username: &str) -> [u8; USERNAME_MAX] {
    let mut buffer = [0u8; USERNAME_MAX];
    let bytes = username.as_bytes();
    for i in 0..bytes.len() {
        buffer[i] = bytes[i];
    }
    buffer
}

fn make_salt(user_id: u32, username: &str) -> u64 {
    let mut salt = 0xA5A5_5A5A_D3C1_B2E0u64 ^ user_id as u64;
    for byte in username.bytes() {
        salt = salt.rotate_left(5) ^ byte as u64;
        salt = salt.wrapping_mul(0x9E37_79B1_85EB_CA87);
    }
    salt
}

fn hash_password(password: &str, salt: u64) -> u64 {
    let mut hash = 0xCBF2_9CE4_8422_2325u64 ^ salt;
    for byte in password.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
        hash ^= hash.rotate_right(29);
    }
    hash ^ salt.rotate_left(17)
}

fn username_as_str(user: &User) -> &str {
    core::str::from_utf8(user.username_bytes()).unwrap_or(DEFAULT_ADMIN_NAME)
}

fn serialize_users(manager: &UserManager, buffer: &mut [u8; BLOCK_SIZE]) -> Result<(), UserError> {
    buffer[0..8].copy_from_slice(USER_DB_MAGIC);
    buffer[8] = USER_DB_VERSION;
    write_u32(buffer, 9, manager.user_count as u32);
    write_u32(buffer, 13, manager.next_id);

    let mut offset = 17;
    for slot in manager.users.iter() {
        if let Some(user) = slot {
            if offset + USER_RECORD_SIZE > BLOCK_SIZE {
                return Err(UserError::CorruptDatabase);
            }

            write_u32(buffer, offset, user.id);
            offset += 4;
            buffer[offset] = user.username_len as u8;
            offset += 1;
            buffer[offset..offset + USERNAME_MAX].copy_from_slice(&user.username);
            offset += USERNAME_MAX;
            write_u64(buffer, offset, user.password_hash);
            offset += 8;
            write_u64(buffer, offset, user.salt);
            offset += 8;
            buffer[offset] = role_to_u8(user.role);
            offset += 1;
            buffer[offset] = status_to_u8(user.status);
            offset += 1;
            buffer[offset] = user.failed_logins;
            offset += 1;
            buffer[offset..offset + 7].fill(0);
            offset += 7;
        }
    }

    Ok(())
}

fn deserialize_users(
    manager: &mut UserManager,
    buffer: &[u8; BLOCK_SIZE],
) -> Result<(), UserError> {
    if &buffer[0..8] != &USER_DB_MAGIC[..] {
        return Err(UserError::CorruptDatabase);
    }
    if buffer[8] != USER_DB_VERSION {
        return Err(UserError::CorruptDatabase);
    }

    let stored_count = read_u32(buffer, 9) as usize;
    if stored_count > MAX_USERS {
        return Err(UserError::CorruptDatabase);
    }

    manager.users = [None; MAX_USERS];
    manager.user_count = 0;
    manager.next_id = read_u32(buffer, 13);
    manager.current_session = None;

    let mut offset = 17;
    for _ in 0..stored_count {
        if offset + USER_RECORD_SIZE > BLOCK_SIZE {
            return Err(UserError::CorruptDatabase);
        }

        let id = read_u32(buffer, offset);
        offset += 4;
        let username_len = buffer[offset] as usize;
        offset += 1;
        if username_len == 0 || username_len > USERNAME_MAX {
            return Err(UserError::CorruptDatabase);
        }

        let mut username = [0u8; USERNAME_MAX];
        username.copy_from_slice(&buffer[offset..offset + USERNAME_MAX]);
        offset += USERNAME_MAX;
        let password_hash = read_u64(buffer, offset);
        offset += 8;
        let salt = read_u64(buffer, offset);
        offset += 8;
        let role = u8_to_role(buffer[offset]).ok_or(UserError::CorruptDatabase)?;
        offset += 1;
        let status = u8_to_status(buffer[offset]).ok_or(UserError::CorruptDatabase)?;
        offset += 1;
        let failed_logins = buffer[offset];
        offset += 1;
        offset += 7;

        let slot = manager
            .users
            .iter_mut()
            .find(|slot| slot.is_none())
            .ok_or(UserError::UserTableFull)?;
        *slot = Some(User {
            id,
            username,
            username_len,
            password_hash,
            salt,
            role,
            status,
            failed_logins,
        });
        manager.user_count += 1;
    }

    if manager.next_id == 0 {
        manager.next_id = 1;
    }

    Ok(())
}

fn write_u32(buffer: &mut [u8; BLOCK_SIZE], offset: usize, value: u32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn read_u32(buffer: &[u8; BLOCK_SIZE], offset: usize) -> u32 {
    u32::from_le_bytes([
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    ])
}

fn write_u64(buffer: &mut [u8; BLOCK_SIZE], offset: usize, value: u64) {
    buffer[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn read_u64(buffer: &[u8; BLOCK_SIZE], offset: usize) -> u64 {
    u64::from_le_bytes([
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
        buffer[offset + 4],
        buffer[offset + 5],
        buffer[offset + 6],
        buffer[offset + 7],
    ])
}

fn role_to_u8(role: UserRole) -> u8 {
    match role {
        UserRole::Admin => 1,
        UserRole::Standard => 2,
    }
}

fn u8_to_role(value: u8) -> Option<UserRole> {
    match value {
        1 => Some(UserRole::Admin),
        2 => Some(UserRole::Standard),
        _ => None,
    }
}

fn status_to_u8(status: UserStatus) -> u8 {
    match status {
        UserStatus::Active => 1,
        UserStatus::Disabled => 2,
        UserStatus::Locked => 3,
    }
}

fn u8_to_status(value: u8) -> Option<UserStatus> {
    match value {
        1 => Some(UserStatus::Active),
        2 => Some(UserStatus::Disabled),
        3 => Some(UserStatus::Locked),
        _ => None,
    }
}
