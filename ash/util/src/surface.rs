use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;

unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &winit::window::Window,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use std::os::raw::c_void;
    use winapi::shared::windef::HWND;
    use winapi::um::libloaderapi::GetModuleHandleW;
    use winit::platform::windows::WindowExtWindows;

    let hwnd = window.hwnd() as HWND;
    let hinstance = GetModuleHandleW(std::ptr::null()) as *const c_void;
    let win32_create_info = vk::Win32SurfaceCreateInfoKHR {
        s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
        p_next: std::ptr::null(),
        flags: Default::default(),
        hinstance,
        hwnd: hwnd as *const c_void,
    };

    let surface_fn = vk::KhrWin32SurfaceFn::load(|name| unsafe {
        std::mem::transmute(entry.get_instance_proc_addr(instance.handle(), name.as_ptr()))
    });

    #[allow(deprecated)]
    let mut surface = std::mem::uninitialized();
    let create_surface_result =
        surface_fn.create_win32_surface_khr(instance.handle(), &win32_create_info, std::ptr::null(), &mut surface);

    match create_surface_result {
        vk::Result::SUCCESS => Ok(surface),
        _ => Err(create_surface_result),
    }
}

pub struct Surface {
    pub loader: ash::extensions::khr::Surface,
    pub vk_surface_khr: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance, window: &winit::window::Window) -> Surface {
        let vk_surface_khr = unsafe { create_surface(entry, instance, window).expect("Failed to create surface") };
        let loader = ash::extensions::khr::Surface::new(entry, instance);

        Surface { loader, vk_surface_khr }
    }
}
