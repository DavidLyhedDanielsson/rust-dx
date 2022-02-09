use windows::{
    Win32::Foundation::*, Win32::System::LibraryLoader::*, Win32::UI::WindowsAndMessaging::*,
};

use crate::window;

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message {
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT::default()
        }
        _ => unsafe { DefWindowProcA(window, message, wparam, lparam) },
    }
}

pub fn create_window(
    name: String,
    width: u32,
    height: u32,
) -> std::result::Result<window::Window, ()> {
    let instance = unsafe { GetModuleHandleA(None) };
    if instance.is_invalid() {
        return Err(());
    }

    let win_name = std::ffi::CString::new(name.to_string()).unwrap();
    let win_class = std::ffi::CString::new("Main").unwrap();

    let wnd_class = WNDCLASSEXA {
        cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        hInstance: instance,
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) },
        lpszClassName: PSTR(win_class.as_ptr() as *mut u8),
        ..Default::default()
    };

    let atom = unsafe { RegisterClassExA(&wnd_class) };
    if atom == 0 {
        return Err(());
    }

    let mut window_rect = RECT {
        left: 0,
        top: 0,
        right: width as i32,
        bottom: height as i32,
    };
    unsafe { AdjustWindowRect(&mut window_rect, WS_OVERLAPPEDWINDOW, false) };

    let hwnd = unsafe {
        CreateWindowExA(
            Default::default(),
            PSTR(win_class.as_ptr() as *mut u8),
            PSTR(win_name.as_ptr() as *mut u8),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            window_rect.right - window_rect.left,
            window_rect.bottom - window_rect.top,
            None, // no parent window
            None, // no menus
            instance,
            std::ptr::null(),
        )
    };

    if hwnd.is_invalid() {
        return Err(());
    }

    unsafe { ShowWindow(hwnd, SW_SHOW) };

    Ok(window::Window {
        name,
        handle: window::handle::Handle::from(hwnd),
        width,
        height,
    })
}
