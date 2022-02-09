use windows::Win32::Foundation::HWND;

#[derive(Copy, Clone)]
pub struct Handle {
    val: HWND
}

impl From<HWND> for Handle {
    fn from(val: HWND) -> Self {
        Handle { val }
    }
}

impl Into<HWND> for Handle {
    fn into(self) -> HWND {
        self.val
    }
}
