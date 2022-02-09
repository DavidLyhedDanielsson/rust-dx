#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;

#[derive(Copy, Clone)]
pub struct Handle {
#[cfg(target_os = "windows")]
    val: HWND
}

#[cfg(target_os = "windows")]
impl From<HWND> for Handle {
    fn from(val: HWND) -> Self {
        Handle { val }
    }
}

#[cfg(target_os = "windows")]
impl Into<HWND> for Handle {
    fn into(self) -> HWND {
        self.val
    }
}