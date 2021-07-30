use ash::version::EntryV1_0;
use ash::version::InstanceV1_0;
use ash::vk;

pub fn c_char_to_string(raw_string_array: &[std::os::raw::c_char]) -> String {
    let raw_string = unsafe { std::ffi::CStr::from_ptr(raw_string_array.as_ptr()) };

    raw_string
        .to_str()
        .expect("Failed to convert c_char to String")
        .to_owned()
}

pub fn is_validation_layer_supported(entry: &ash::Entry) -> bool {
    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("Failed to enumerate instance layer properties");

    if layer_properties.is_empty() {
        eprintln!("No available layers");
        return false;
    }

    let validation_layer_name = String::from("VK_LAYER_KHRONOS_validation");
    let mut layer_found = false;
    for layer_property in layer_properties.iter() {
        let layer_name = crate::utility::c_char_to_string(&layer_property.layer_name);
        if *validation_layer_name == layer_name {
            layer_found = true;
            break;
        }
    }

    layer_found
}

pub fn get_graphics_queue_family_index(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> Option<u32> {    
    let queue_families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    for (index, queue_family) in queue_families.iter().enumerate() {
        if queue_family.queue_count > 0 && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            return Some(index as u32);
        }
    }
    return None;
}
