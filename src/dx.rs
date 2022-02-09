use crate::window::Window;
use windows::{
    Win32::Foundation::BOOL,
    Win32::{
        Graphics::{Direct3D::*, Direct3D11::*, Dxgi::*},
    },
    Win32::{Foundation::{PSTR}, Graphics::Dxgi::Common::*}, core::Interface,
};

use std::{
    ffi::c_void,
    fs::File,
    io::Read,
    mem::{size_of_val},
    result::Result,
};

pub fn create_device(
    window: &Window,
) -> Result<(ID3D11Device, ID3D11DeviceContext, IDXGISwapChain), ()> {
    let adapter = unsafe {
        let factory = { CreateDXGIFactory::<IDXGIFactory>().unwrap() };

        // Syntax is a bit ugly because functions like GetDesc returns a Result
        (0..8) // Think anyone has 8 adapters available?
            .flat_map(|i| factory.EnumAdapters(i))
            .flat_map(|adapter: IDXGIAdapter| adapter.GetDesc().map(|desc| (adapter, desc)))
            .filter(|(_, desc)| {
                str::to_ascii_lowercase(
                    &String::from_utf16(&desc.Description).unwrap_or(String::new()),
                )
                .contains("nvidia")
            })
            .map(|(adapter, _)| adapter)
            .next()
    };

    let flags = if cfg!(debug_assertions) {
        D3D11_CREATE_DEVICE_DEBUG
    } else {
        0
    };

    let feature_levels = [D3D_FEATURE_LEVEL_11_1];

   let driver_type = if adapter.is_some() {
       D3D_DRIVER_TYPE_UNKNOWN
   } else {
       D3D_DRIVER_TYPE_HARDWARE
   };

    let mut device: Option<ID3D11Device> = None;
    let mut context: Option<ID3D11DeviceContext> = None;

    let device_result = unsafe {
        D3D11CreateDevice(
            adapter,
            driver_type,
            None,
            flags,
            &feature_levels as *const i32,
            1,
            D3D11_SDK_VERSION,
            &mut device,
            std::ptr::null_mut(),
            &mut context,
        ) 
    };

    if let Err(val) = device_result {
        let err_str = val.to_string();
        println!("Error when creating device: {}", err_str);
    }

    let device = device.unwrap();

    let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
        Width: window.width,
        Height: window.height,
        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
        Stereo: BOOL(0),
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: 1,
        Scaling: DXGI_MODE_SCALING_UNSPECIFIED,
        SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
        AlphaMode: DXGI_ALPHA_MODE_UNSPECIFIED,
        Flags: 0,
    };

    let factory = unsafe { CreateDXGIFactory2::<IDXGIFactory2>(0).unwrap() };
    let swap_chain = unsafe { factory.CreateSwapChainForHwnd(&device, Some(window.handle.into()), &swap_chain_desc, std::ptr::null_mut(), None) };

    Ok((device, context.unwrap(), swap_chain.unwrap().cast::<IDXGISwapChain>().unwrap()))
}

pub fn create_backbuffer_rtv(
    device: &ID3D11Device,
    swap_chain: &IDXGISwapChain,
) -> Result<ID3D11RenderTargetView, ()> {
    let back_buffer: ID3D11Texture2D = unsafe { swap_chain.GetBuffer(0) }.unwrap();

    let rtv = unsafe { device.CreateRenderTargetView(back_buffer, std::ptr::null()) }.unwrap();

    Ok(rtv)
}

pub fn create_depth_stencil_view(
    device: &ID3D11Device,
    window: &Window,
) -> Result<ID3D11DepthStencilView, ()> {
    let texture_desc = D3D11_TEXTURE2D_DESC {
        Width: window.width,
        Height: window.height,
        MipLevels: 0,
        ArraySize: 1,
        Format: DXGI_FORMAT_D32_FLOAT,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_DEPTH_STENCIL,
        CPUAccessFlags: 0,
        MiscFlags: 0,
    };

    let texture = unsafe { device.CreateTexture2D(&texture_desc, std::ptr::null()) }.unwrap();

    let depth_stencil_view =
        unsafe { device.CreateDepthStencilView(texture, std::ptr::null_mut()) }.unwrap();

    Ok(depth_stencil_view)
}

pub fn create_input_layout(
    device: &ID3D11Device,
    shader_byte_code: &mut Vec<u8>,
) -> Result<ID3D11InputLayout, ()> {
    let mut name = "POSITION\0".to_string();

    let input_desc = D3D11_INPUT_ELEMENT_DESC {
        SemanticName: PSTR(name.as_mut_ptr()),
        SemanticIndex: 0,
        Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
        InputSlot: 0,
        AlignedByteOffset: 0,
        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
        InstanceDataStepRate: 0,
    };

    let res = unsafe {
        device.CreateInputLayout(
            &input_desc,
            1,
            shader_byte_code.as_mut_ptr() as *mut c_void,
            shader_byte_code.len(),
        )
    }
    .unwrap();

    Ok(res)
}

pub fn create_vertex_buffer(device: &ID3D11Device) -> Result<ID3D11Buffer, ()> {
    let mut vertex_data = [
        0.0_f32, -1.0_f32, 0.0_f32, 1.0_f32, -1.0_f32, 1.0_f32, 0.0_f32, 1.0_f32, 1.0_f32, 1.0_f32,
        0.0_f32, 1.0_f32,
    ];

    let desc = D3D11_BUFFER_DESC {
        ByteWidth: size_of_val(&vertex_data) as u32,
        Usage: D3D11_USAGE_IMMUTABLE,
        BindFlags: D3D11_BIND_VERTEX_BUFFER,
        CPUAccessFlags: 0,
        MiscFlags: 0,
        StructureByteStride: 0,
    };

    let initial_data = D3D11_SUBRESOURCE_DATA {
        pSysMem: vertex_data.as_mut_ptr() as *mut c_void,
        SysMemPitch: 0,
        SysMemSlicePitch: 0,
    };

    let res = unsafe { device.CreateBuffer(&desc, &initial_data) }.unwrap();

    Ok(res)
}

pub fn create_shaders(
    device: &ID3D11Device,
) -> Result<(ID3D11VertexShader, Vec<u8>, ID3D11PixelShader), ()> {
    let mut file = File::open("vertex_shader.cso").unwrap();
    let metadata = std::fs::metadata("vertex_shader.cso").unwrap();
    let mut vertex_shader_buffer = vec![0; metadata.len() as usize];
    file.read(&mut vertex_shader_buffer);

    let v = unsafe {
        device.CreateVertexShader(
            vertex_shader_buffer.as_mut_ptr() as *mut c_void,
            metadata.len() as usize,
            None,
        )
    }
    .unwrap();

    let mut file = File::open("pixel_shader.cso").unwrap();
    let metadata = std::fs::metadata("pixel_shader.cso").unwrap();
    let mut buffer = vec![0; metadata.len() as usize];
    file.read(&mut buffer);

    let p = unsafe {
        device.CreatePixelShader(
            buffer.as_mut_ptr() as *mut c_void,
            metadata.len() as usize,
            None,
        )
    }
    .unwrap();

    Ok((v, vertex_shader_buffer, p))
}
