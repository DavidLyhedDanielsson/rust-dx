use super::Window;

#[cfg(target_os = "windows")]
mod windows;

pub struct CreateWindowParams
{
    pub name: String,
    pub width: u32,
    pub height: u32,
}

pub fn create_window(params: CreateWindowParams) -> Window {
    windows::create_window(params.name, params.width, params.height).unwrap()
}