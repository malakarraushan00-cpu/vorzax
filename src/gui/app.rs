/// Application Framework
/// 
/// Application lifecycle management and inter-process communication

use spin::Mutex;
use crate::gui::window::WindowId;

/// Application state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Starting,
    Running,
    Paused,
    Stopped,
}

/// Application instance
pub struct Application {
    pub app_id: u32,
    pub name: [u8; 256],
    pub name_len: usize,
    pub state: AppState,
    pub main_window: Option<WindowId>,
    pub process_id: u64,
}

impl Application {
    pub fn new(app_id: u32, name: &str) -> Self {
        let mut app_name = [0u8; 256];
        let bytes = name.as_bytes();
        let len = core::cmp::min(bytes.len(), 256);
        for i in 0..len {
            app_name[i] = bytes[i];
        }
        
        Application {
            app_id,
            name: app_name,
            name_len: len,
            state: AppState::Starting,
            main_window: None,
            process_id: 0,
        }
    }
    
    pub fn run(&mut self) {
        self.state = AppState::Running;
    }
    
    pub fn pause(&mut self) {
        self.state = AppState::Paused;
    }
    
    pub fn stop(&mut self) {
        self.state = AppState::Stopped;
    }
}

/// Application registry
pub struct AppRegistry {
    apps: [Option<Application>; 32],
    app_count: usize,
    next_id: u32,
}

impl AppRegistry {
    pub fn new() -> Self {
        AppRegistry {
            apps: [None; 32],
            app_count: 0,
            next_id: 1,
        }
    }
    
    pub fn register_app(&mut self, name: &str) -> Option<u32> {
        if self.app_count < 32 {
            let id = self.next_id;
            self.next_id += 1;
            
            let app = Application::new(id, name);
            
            if let Some(slot) = self.apps.iter_mut().find(|s| s.is_none()) {
                *slot = Some(app);
                self.app_count += 1;
                return Some(id);
            }
        }
        None
    }
    
    pub fn unregister_app(&mut self, app_id: u32) {
        if let Some(pos) = self.apps.iter().position(|app| {
            if let Some(a) = app {
                return a.app_id == app_id;
            }
            false
        }) {
            self.apps[pos] = None;
            self.app_count -= 1;
        }
    }
    
    pub fn get_app(&self, app_id: u32) -> Option<&Application> {
        self.apps.iter()
            .find_map(|slot| {
                if let Some(app) = slot {
                    if app.app_id == app_id {
                        return Some(app);
                    }
                }
                None
            })
    }
    
    pub fn get_app_mut(&mut self, app_id: u32) -> Option<&mut Application> {
        self.apps.iter_mut()
            .find_map(|slot| {
                if let Some(app) = slot {
                    if app.app_id == app_id {
                        return Some(app);
                    }
                }
                None
            })
    }
    
    pub fn launch_app(&mut self, app_id: u32) {
        if let Some(app) = self.get_app_mut(app_id) {
            app.run();
        }
    }
}

static APP_REGISTRY: Mutex<AppRegistry> = Mutex::new(AppRegistry {
    apps: [None; 32],
    app_count: 0,
    next_id: 1,
});

pub fn init() {
    // Initialize application framework
}

pub fn register_app(name: &str) -> Option<u32> {
    let mut registry = APP_REGISTRY.lock();
    registry.register_app(name)
}

pub fn launch_app(app_id: u32) {
    let mut registry = APP_REGISTRY.lock();
    registry.launch_app(app_id);
}

pub fn get_app_state(app_id: u32) -> Option<AppState> {
    let registry = APP_REGISTRY.lock();
    registry.get_app(app_id).map(|app| app.state)
}

pub fn set_app_window(app_id: u32, window_id: WindowId) {
    let mut registry = APP_REGISTRY.lock();
    if let Some(app) = registry.get_app_mut(app_id) {
        app.main_window = Some(window_id);
    }
}

pub fn close_app(app_id: u32) {
    let mut registry = APP_REGISTRY.lock();
    registry.unregister_app(app_id);
}
