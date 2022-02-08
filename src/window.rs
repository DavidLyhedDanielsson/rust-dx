use windows::Win32::Foundation::HWND;

pub struct Window {
    pub name: String,
    pub hwnd: HWND,
    pub width: u32,
    pub height: u32,
}
