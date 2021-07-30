use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;

pub fn create_logical_device(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> (ash::Device, vk::Queue) {
    let queue_familty_index = crate::utility::get_graphics_queue_family_index(instance, physical_device).unwrap();

    let queue_priorities = [1.0_f32];

    let queue_create_info = vk::DeviceQueueCreateInfo {
        s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DeviceQueueCreateFlags::empty(),
        queue_family_index: queue_familty_index,
        p_queue_priorities: queue_priorities.as_ptr(),
        queue_count: queue_priorities.len() as u32,
    };

    let physical_device_features = vk::PhysicalDeviceFeatures { ..Default::default() };

    let layers = vec!["VK_LAYER_KHRONOS_validation"];
    let layers_raw: Vec<std::ffi::CString> = layers
        .iter()
        .map(|layer_name| std::ffi::CString::new(*layer_name).unwrap())
        .collect();
    let layers_ptr: Vec<*const i8> = layers_raw.iter().map(|layer_name| layer_name.as_ptr()).collect();

    let device_create_info = vk::DeviceCreateInfo {
        s_type: vk::StructureType::DEVICE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DeviceCreateFlags::empty(),
        queue_create_info_count: 1,
        p_queue_create_infos: &queue_create_info,
        enabled_layer_count: layers_ptr.len() as u32,
        pp_enabled_layer_names: layers_ptr.as_ptr(),
        enabled_extension_count: 0,
        pp_enabled_extension_names: std::ptr::null(),
        p_enabled_features: &physical_device_features,
    };

    let device: ash::Device = unsafe {
        instance
            .create_device(physical_device, &device_create_info, None)
            .expect("Failed to create device")
    };

    let graphics_queue = unsafe { device.get_device_queue(queue_familty_index, 0) };

    (device, graphics_queue)
}
