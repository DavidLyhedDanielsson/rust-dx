use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D::Fxc::*, Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D11, Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*,
    Win32::System::LibraryLoader::*, Win32::System::Threading::*,
    Win32::System::WindowsProgramming::*, Win32::UI::WindowsAndMessaging::*,
};

struct Window {
    pub name: String,
    pub hwnd: HWND,
}

extern "system" fn wndproc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT::default()
        }
        _ => {
            unsafe { DefWindowProcA(window, message, wparam, lparam) }
        }
    }
}

fn create_window(name: &str) -> std::result::Result<Window, ()> {
    let instance = unsafe { GetModuleHandleA(None) };
    if instance.is_invalid() {
        return Err(());
    }

    let name = name.to_string();
    let mut class = "RustD3D11Class\0".to_string();

    let wnd_class = WNDCLASSEXA {
        cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        hInstance: instance,
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) },
        lpszClassName: PSTR(class.as_mut_ptr()),
        ..Default::default()
    };

    let atom = unsafe { RegisterClassExA(&wnd_class) };
    if atom == 0 {
        return Err(());
    }

    let mut window_rect = RECT {
        left: 0,
        top: 0,
        right: 1280,
        bottom: 720,
    };
    unsafe { AdjustWindowRect(&mut window_rect, WS_OVERLAPPEDWINDOW, false) };

    let hwnd = unsafe {
        CreateWindowExA(
            Default::default(),
            class,
            name.to_string(),
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

    Ok(Window { name, hwnd })
}

fn main() {
    let window = create_window("Test window\0").unwrap();

    loop {
        let mut message = MSG::default();
        if unsafe { PeekMessageA(&mut message, None, 0, 0, PM_REMOVE)}.into() {
            unsafe {
                TranslateMessage(&message);
                DispatchMessageA(&message);
            }

            if message.message == WM_QUIT {
                break;
            }
        }
    }
}
