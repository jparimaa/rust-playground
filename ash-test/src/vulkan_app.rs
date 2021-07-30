use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;

pub struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,
    debug_utils: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    _graphics_queue: vk::Queue,
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.debug_utils
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.instance.destroy_instance(None);
        }
    }
}

impl VulkanApp {
    pub fn new() -> VulkanApp {
        let entry = ash::Entry::new().unwrap();

        if !crate::utility::is_validation_layer_supported(&entry) {
            panic!("Validation layers not supported");
        }

        let instance = crate::instance::create_instance(&entry);
        let (debug_utils, debug_messenger) = crate::instance::create_debug_utils(&entry, &instance);
        let physical_device = crate::physical_device::get_physical_device(&instance);

        let (device, graphics_queue) = crate::device::create_logical_device(&instance, physical_device);

        VulkanApp {
            _entry: entry,
            instance,
            debug_utils,
            debug_messenger,
            physical_device,
            device,
            _graphics_queue: graphics_queue,
        }
    }

    pub fn draw_frame(&mut self) {
        // Drawing will be here
    }
}
