/// Boot module - UEFI firmware interface and initialization
pub mod entry;

pub fn init() {
    entry::init_memory();
}
