use ash::version::InstanceV1_0;
use ash::vk;

pub fn create_logical_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    indices: &crate::queue_family::QueueFamilyIndices,
) -> ash::Device {
    use std::collections::HashSet;
    let mut unique_queue_indices = HashSet::new();
    unique_queue_indices.insert(indices.graphics_family.unwrap());
    unique_queue_indices.insert(indices.present_family.unwrap());

    let queue_priorities = [1.0_f32];
    let mut queue_create_infos = vec![];

    for queue_family_index in unique_queue_indices {
        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: queue_family_index,
            p_queue_priorities: queue_priorities.as_ptr(),
            queue_count: queue_priorities.len() as u32,
        };
        queue_create_infos.push(queue_create_info);
    }

    let physical_device_features = vk::PhysicalDeviceFeatures { ..Default::default() };

    let layers = vec!["VK_LAYER_KHRONOS_validation"];
    let layers_raw: Vec<std::ffi::CString> = layers
        .iter()
        .map(|layer_name| std::ffi::CString::new(*layer_name).unwrap())
        .collect();
    let layers_ptr: Vec<*const i8> = layers_raw.iter().map(|layer_name| layer_name.as_ptr()).collect();

    let enable_extension_names = [ash::extensions::khr::Swapchain::name().as_ptr()];

    let device_create_info = vk::DeviceCreateInfo {
        s_type: vk::StructureType::DEVICE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DeviceCreateFlags::empty(),
        queue_create_info_count: queue_create_infos.len() as u32,
        p_queue_create_infos: queue_create_infos.as_ptr(),
        enabled_layer_count: layers_ptr.len() as u32,
        pp_enabled_layer_names: layers_ptr.as_ptr(),
        enabled_extension_count: enable_extension_names.len() as u32,
        pp_enabled_extension_names: enable_extension_names.as_ptr(),
        p_enabled_features: &physical_device_features,
    };

    return unsafe {
        instance
            .create_device(physical_device, &device_create_info, None)
            .expect("Failed to create device")
    };
}
