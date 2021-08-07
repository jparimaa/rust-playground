use ash::version::EntryV1_0;
use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new() -> QueueFamilyIndices {
        QueueFamilyIndices { graphics_family: None, present_family: None }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub fn get_queue_family_indices(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface: &crate::surface::Surface,
) -> QueueFamilyIndices {
    let queue_families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
    let mut indices = QueueFamilyIndices::new();

    for (index, queue_family) in queue_families.iter().enumerate() {
        if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            indices.graphics_family = Some(index as u32);
        }

        let is_present_support = unsafe {
            surface
                .loader
                .get_physical_device_surface_support(physical_device, index as u32, surface.vk_surface_khr)
        };

        if is_present_support {
            indices.present_family = Some(index as u32);
        }

        if indices.is_complete() {
            break;
        }
    }
    return indices;
}

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

pub fn are_device_extensions_supported(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    extensions: &std::collections::HashSet<String>,
) -> bool {
    let available_extensions = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device)
            .expect("Failed to get device extension properties")
    };

    let available_extension_names = available_extensions
        .iter()
        .map(|ext| crate::utility::c_char_to_string(&ext.extension_name))
        .collect::<std::vec::Vec<String>>();

    let mut required_extensions = extensions.clone();

    for extension_name in available_extension_names.iter() {
        required_extensions.remove(extension_name);
    }

    return required_extensions.is_empty();
}

pub fn read_file(filepath: &std::path::Path) -> Vec<u8> {
    use std::io::Read;
    let spv_file = std::fs::File::open(filepath).expect(&format!("Failed to load spv file at {:?}", filepath));
    spv_file.bytes().filter_map(|byte| byte.ok()).collect()
}

pub fn create_shader_module(device: &ash::Device, byte_code: Vec<u8>) -> vk::ShaderModule {
    let shader_module_create_info = vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: byte_code.len(),
        p_code: byte_code.as_ptr() as *const u32,
    };

    unsafe {
        device
            .create_shader_module(&shader_module_create_info, None)
            .expect("Failed to create shader module")
    }
}
