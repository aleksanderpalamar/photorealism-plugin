use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Direct3D::{
    D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL, D3D_FEATURE_LEVEL_10_0, D3D_FEATURE_LEVEL_10_1,
    D3D_FEATURE_LEVEL_11_0,
};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_SDK_VERSION, D3D11CreateDeviceAndSwapChain, ID3D11Device, ID3D11DeviceContext,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_MODE_DESC, DXGI_RATIONAL, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::{
    DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGISwapChain,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, UnregisterClassW, WNDCLASSW,
    WS_OVERLAPPEDWINDOW,
};
use windows::core::PCWSTR;

pub struct Probe {
    pub swap_chain: IDXGISwapChain,
    window: HWND,
    instance: HINSTANCE,
}

impl Probe {
    pub fn create() -> windows::core::Result<Self> {
        let module = unsafe { GetModuleHandleW(None)? };
        let instance = HINSTANCE(module.0);
        let class_name: Vec<u16> = "PhotorealismRustProbe\0".encode_utf16().collect();
        let window = create_window(instance, &class_name)?;
        let swap_chain = create_swap_chain(window)?;
        Ok(Self {
            swap_chain,
            window,
            instance,
        })
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        let name: Vec<u16> = "PhotorealismRustProbe\0".encode_utf16().collect();
        let _ = unsafe { DestroyWindow(self.window) };
        let _ = unsafe { UnregisterClassW(PCWSTR(name.as_ptr()), Some(self.instance)) };
    }
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(window, message, wparam, lparam) }
}

fn create_window(instance: HINSTANCE, name: &[u16]) -> windows::core::Result<HWND> {
    let class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hInstance: instance,
        lpszClassName: PCWSTR(name.as_ptr()),
        ..Default::default()
    };
    unsafe { RegisterClassW(&class) };
    unsafe {
        CreateWindowExW(
            Default::default(),
            PCWSTR(name.as_ptr()),
            PCWSTR(name.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            64,
            64,
            None,
            None,
            Some(instance),
            None,
        )
    }
}

fn create_swap_chain(window: HWND) -> windows::core::Result<IDXGISwapChain> {
    let description = description(window);
    let levels = [
        D3D_FEATURE_LEVEL_11_0,
        D3D_FEATURE_LEVEL_10_1,
        D3D_FEATURE_LEVEL_10_0,
    ];
    let mut swap_chain = None;
    let mut device: Option<ID3D11Device> = None;
    let mut context: Option<ID3D11DeviceContext> = None;
    let mut level = D3D_FEATURE_LEVEL::default();
    unsafe {
        D3D11CreateDeviceAndSwapChain(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            Default::default(),
            Default::default(),
            Some(&levels),
            D3D11_SDK_VERSION,
            Some(&description),
            Some(&mut swap_chain),
            Some(&mut device),
            Some(&mut level),
            Some(&mut context),
        )?;
    }
    swap_chain.ok_or_else(windows::core::Error::from_win32)
}

fn description(window: HWND) -> DXGI_SWAP_CHAIN_DESC {
    DXGI_SWAP_CHAIN_DESC {
        BufferDesc: DXGI_MODE_DESC {
            Width: 64,
            Height: 64,
            RefreshRate: DXGI_RATIONAL::default(),
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            ..Default::default()
        },
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: 1,
        OutputWindow: window,
        Windowed: true.into(),
        SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
        ..Default::default()
    }
}
