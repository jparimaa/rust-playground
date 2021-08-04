use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;

pub struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,
    debug_utils: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    surface: crate::surface::Surface,
    _physical_device: vk::PhysicalDevice,
    device: ash::Device,
    _queue_families: crate::utility::QueueFamilyIndices,
    swapchain: crate::swapchain::Swapchain,
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            self.swapchain.destroy(&self.device);
            self.device.destroy_device(None);
            self.surface.loader.destroy_surface(self.surface.vk_surface_khr, None);
            self.debug_utils
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.instance.destroy_instance(None);
        }
    }
}

impl VulkanApp {
    pub fn new(window: &winit::window::Window) -> VulkanApp {
        let entry = ash::Entry::new().unwrap();

        if !crate::utility::is_validation_layer_supported(&entry) {
            panic!("Validation layers not supported");
        }

        let instance = crate::instance::create_instance(&entry);
        let (debug_utils, debug_messenger) = crate::instance::create_debug_utils(&entry, &instance);
        let surface = crate::surface::Surface::new(&entry, &instance, window);
        let (physical_device, queue_families) = crate::physical_device::get_physical_device(&instance, &surface);
        let device = crate::device::create_logical_device(&instance, physical_device, &queue_families);
        let swapchain = crate::swapchain::Swapchain::new(&instance, &device, physical_device, &surface);
        create_graphics_pipeline(&device);

        VulkanApp {
            _entry: entry,
            instance,
            debug_utils,
            debug_messenger,
            surface,
            _physical_device: physical_device,
            device,
            _queue_families: queue_families,
            swapchain,
        }
    }

    pub fn draw_frame(&mut self) {
        // Drawing will be here
    }
}

fn create_graphics_pipeline(device: &ash::Device) {
    use std::path::Path;
    let vertex_shader_code = crate::utility::read_file(Path::new("ash-test/shader_bin/vert.spv"));
    let fragment_shader_code = crate::utility::read_file(Path::new("ash-test/shader_bin/frag.spv"));

    let vertex_shader_module = crate::utility::create_shader_module(device, vertex_shader_code);
    let fragment_shader_module = crate::utility::create_shader_module(device, fragment_shader_code);

    let main_function_name = std::ffi::CString::new("main").unwrap();

    let _shader_stages = [
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            module: vertex_shader_module,
            p_name: main_function_name.as_ptr(),
            p_specialization_info: std::ptr::null(),
            stage: vk::ShaderStageFlags::VERTEX,
        },
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            module: fragment_shader_module,
            p_name: main_function_name.as_ptr(),
            p_specialization_info: std::ptr::null(),
            stage: vk::ShaderStageFlags::FRAGMENT,
        },
    ];

    unsafe {
        device.destroy_shader_module(vertex_shader_module, None);
        device.destroy_shader_module(fragment_shader_module, None);
    }
}
