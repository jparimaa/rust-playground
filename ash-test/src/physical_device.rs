use ash::version::InstanceV1_0;
use ash::vk;

pub fn get_physical_device(instance: &ash::Instance) -> vk::PhysicalDevice {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices")
    };
    
    for physical_device in physical_devices {
        if crate::utility::get_graphics_queue_family_index(instance, physical_device).is_some() {
            return physical_device;
        }
    }

    panic!("Failed to find a suitable GPU");
}

