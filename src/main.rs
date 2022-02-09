use std::{alloc::Layout, mem, ffi::CStr};

use dx::{
    create_backbuffer_rtv, create_depth_stencil_view, create_device, create_input_layout,
    create_shaders, create_vertex_buffer,
};
use window::platform::{CreateWindowParams, create_window};
use windows::{
    core::*,
    Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D11::{ID3D11Debug, D3D11_VIEWPORT},
    Win32::{Graphics::Direct3D11::{ID3D11InfoQueue, D3D11_MESSAGE}},
    Win32::UI::WindowsAndMessaging::*,
};

mod dx;
mod window;

fn main() {
    let params = CreateWindowParams {
        name: "Test window".to_string(),
        width: 1280,
        height: 720,
    };

    let window = create_window(params);
    let (device, context, swap_chain) = create_device(&window).unwrap();
    let rtv = create_backbuffer_rtv(&device, &swap_chain).unwrap();
    let dsv = create_depth_stencil_view(&device, &window).unwrap();

    let viewport = D3D11_VIEWPORT {
        TopLeftX: 0.0,
        TopLeftY: 0.0,
        Width: window.width as f32,
        Height: window.height as f32,
        MinDepth: 0.0,
        MaxDepth: 1.0,
    };
    unsafe { context.RSSetViewports(1, &viewport) };

    let vertex_buffer = create_vertex_buffer(&device).unwrap();
    let (vertex_shader, mut vertex_shader_byte_code, pixel_shader) =
        create_shaders(&device).unwrap();
    let input_layout = create_input_layout(&device, &mut vertex_shader_byte_code).unwrap();

    unsafe { context.OMSetRenderTargets(1, &Some(rtv.cast().unwrap()), Some(dsv.cast().unwrap())) };
    unsafe { context.IASetVertexBuffers(0, 1, &Some(vertex_buffer), &16, &0) };
    unsafe { context.IASetInputLayout(input_layout) };
    unsafe { context.PSSetShader(pixel_shader, &None, 0) };
    unsafe { context.VSSetShader(vertex_shader, &None, 0) };
    unsafe { context.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST) };

    let debug = device.cast::<ID3D11Debug>().unwrap();
    let queue = debug.cast::<ID3D11InfoQueue>().unwrap();
    unsafe { queue.PushEmptyStorageFilter() }.unwrap();

    loop {
        let mut message = MSG::default();
        if unsafe { PeekMessageA(&mut message, None, 0, 0, PM_REMOVE) }.into() {
            unsafe {
                TranslateMessage(&message);
                DispatchMessageA(&message);
            }

            if message.message == WM_QUIT {
                break;
            }
        }

        let message_count = unsafe { queue.GetNumStoredMessages() };
        for i in 0..message_count {
            let mut message_size: usize = 0;
            unsafe { queue.GetMessage(i, std::ptr::null_mut(), &mut message_size) }.unwrap();

            let layout = Layout::from_size_align(message_size, mem::align_of::<u8>()).unwrap();

            let message: *mut D3D11_MESSAGE = unsafe {std::alloc::alloc_zeroed(layout)} as *mut D3D11_MESSAGE;
            unsafe { queue.GetMessage(i, message as *mut D3D11_MESSAGE, &mut message_size) }.unwrap();

            let message_string = unsafe { CStr::from_ptr((*message).pDescription as *const i8) } ;

            println!("DX MESSAGE: {:?}", message_string);

            unsafe { std::alloc::dealloc(message as *mut u8, layout) };
        }
        unsafe { queue.ClearStoredMessages() };

        let mut clear_color = [0.2_f32, 0.2_f32, 0.2_f32, 1.0_f32];
        unsafe { context.ClearRenderTargetView(&rtv, clear_color.as_mut_ptr()) };
        unsafe { context.ClearDepthStencilView(&dsv, 0xffffffff, 1_f32, 0) };

        unsafe { context.Draw(3, 0) };

        unsafe { swap_chain.Present(0, 0) }.unwrap();
    }
}
