use ash::version::InstanceV1_0;
use ash::vk;

pub fn get_physical_device(
    instance: &ash::Instance,
    surface: &crate::surface::Surface,
) -> (vk::PhysicalDevice, crate::utility::QueueFamilyIndices) {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices")
    };
    for physical_device in physical_devices {
        let indices = crate::utility::get_queue_family_indices(instance, physical_device, surface);
        if indices.is_complete() {
            return (physical_device, indices);
        }
    }

    panic!("Failed to find a suitable GPU");
}
