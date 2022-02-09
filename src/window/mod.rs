pub mod handle;

pub struct Window {
    pub name: String,
    pub handle: handle::Handle,
    pub width: u32,
    pub height: u32,
}
