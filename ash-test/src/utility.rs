use ash::version::DeviceV1_0;
use ash::version::EntryV1_0;
use ash::version::InstanceV1_0;
use ash::vk;

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new() -> QueueFamilyIndices {
        QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        }
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

pub fn create_buffer(
    device: &ash::Device,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    required_memory_properties: vk::MemoryPropertyFlags,
    device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_create_info = vk::BufferCreateInfo {
        s_type: vk::StructureType::BUFFER_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::BufferCreateFlags::empty(),
        size,
        usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        queue_family_index_count: 0,
        p_queue_family_indices: std::ptr::null(),
    };

    let buffer = unsafe {
        device
            .create_buffer(&buffer_create_info, None)
            .expect("Failed to create vertex buffer")
    };

    let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
    let memory_type = find_memory_type(
        mem_requirements.memory_type_bits,
        required_memory_properties,
        device_memory_properties,
    );

    let allocate_info = vk::MemoryAllocateInfo {
        s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
        p_next: std::ptr::null(),
        allocation_size: mem_requirements.size,
        memory_type_index: memory_type,
    };

    let buffer_memory = unsafe {
        device
            .allocate_memory(&allocate_info, None)
            .expect("Failed to allocate vertex buffer memory!")
    };

    unsafe {
        device
            .bind_buffer_memory(buffer, buffer_memory, 0)
            .expect("Failed to bind buffer");
    }

    (buffer, buffer_memory)
}

pub fn copy_buffer(
    device: &ash::Device,
    submit_queue: vk::Queue,
    command_pool: vk::CommandPool,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
) {
    let allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: std::ptr::null(),
        command_buffer_count: 1,
        command_pool,
        level: vk::CommandBufferLevel::PRIMARY,
    };

    let command_buffers = unsafe {
        device
            .allocate_command_buffers(&allocate_info)
            .expect("Failed to allocate command buffer")
    };
    let command_buffer = command_buffers[0];

    let begin_info = vk::CommandBufferBeginInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
        p_next: std::ptr::null(),
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        p_inheritance_info: std::ptr::null(),
    };

    unsafe {
        device
            .begin_command_buffer(command_buffer, &begin_info)
            .expect("Failed to begin command buffer");

        let copy_regions = [vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        }];

        device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &copy_regions);

        device
            .end_command_buffer(command_buffer)
            .expect("Failed to end command buffer");
    }

    let submit_info = [vk::SubmitInfo {
        s_type: vk::StructureType::SUBMIT_INFO,
        p_next: std::ptr::null(),
        wait_semaphore_count: 0,
        p_wait_semaphores: std::ptr::null(),
        p_wait_dst_stage_mask: std::ptr::null(),
        command_buffer_count: 1,
        p_command_buffers: &command_buffer,
        signal_semaphore_count: 0,
        p_signal_semaphores: std::ptr::null(),
    }];

    unsafe {
        device
            .queue_submit(submit_queue, &submit_info, vk::Fence::null())
            .expect("Failed to submit queue");
        device.queue_wait_idle(submit_queue).expect("Failed to wait queue idle");

        device.free_command_buffers(command_pool, &command_buffers);
    }
}

pub fn find_memory_type(
    type_filter: u32,
    required_properties: vk::MemoryPropertyFlags,
    mem_properties: &vk::PhysicalDeviceMemoryProperties,
) -> u32 {
    for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
        if (type_filter & (1 << i)) > 0 && memory_type.property_flags.contains(required_properties) {
            return i as u32;
        }
    }

    panic!("Failed to find suitable memory type")
}

pub fn create_command_pool(device: &ash::Device, queue_family_index: u32) -> vk::CommandPool {
    let command_pool_create_info = vk::CommandPoolCreateInfo {
        s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::CommandPoolCreateFlags::empty(),
        queue_family_index,
    };

    unsafe {
        device
            .create_command_pool(&command_pool_create_info, None)
            .expect("Failed to create command pool")
    }
}
