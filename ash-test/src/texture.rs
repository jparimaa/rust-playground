use ash::version::DeviceV1_0;
use ash::vk;

pub struct Texture {
    image: vk::Image,
    memory: vk::DeviceMemory,
}

impl Texture {
    pub fn new(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
        image_path: &std::path::Path,
    ) -> Texture {
        let image_file = crate::image_file::ImageFile::new(image_path);

        let (image, memory) = create_image(device, &image_file, device_memory_properties);

        let (staging_buffer, staging_buffer_memory) = crate::utility::create_buffer(
            device,
            image_file.size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device_memory_properties,
        );

        unsafe {
            let data_ptr = device
                .map_memory(staging_buffer_memory, 0, image_file.size, vk::MemoryMapFlags::empty())
                .expect("Failed to map memory") as *mut u8;
            data_ptr.copy_from_nonoverlapping(image_file.data.as_ptr(), image_file.data.len());
            device.unmap_memory(staging_buffer_memory);
        }

        crate::utility::transition_image_layout(
            device,
            command_pool,
            submit_queue,
            image,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        );

        crate::utility::copy_buffer_to_image(
            device,
            command_pool,
            submit_queue,
            staging_buffer,
            image,
            image_file.width,
            image_file.height,
        );

        crate::utility::transition_image_layout(
            device,
            command_pool,
            submit_queue,
            image,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );

        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        Texture { image, memory }
    }

    pub fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.destroy_image(self.image, None);
            device.free_memory(self.memory, None);
        }
    }
}

fn create_image(
    device: &ash::Device,
    image_file: &crate::image_file::ImageFile,
    device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
) -> (vk::Image, vk::DeviceMemory) {
    let image_create_info = vk::ImageCreateInfo {
        s_type: vk::StructureType::IMAGE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ImageCreateFlags::empty(),
        image_type: vk::ImageType::TYPE_2D,
        format: vk::Format::R8G8B8A8_UNORM,
        extent: vk::Extent3D {
            width: image_file.width,
            height: image_file.height,
            depth: 1,
        },
        mip_levels: 1,
        array_layers: 1,
        samples: vk::SampleCountFlags::TYPE_1,
        tiling: vk::ImageTiling::OPTIMAL,
        usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        queue_family_index_count: 0,
        p_queue_family_indices: std::ptr::null(),
        initial_layout: vk::ImageLayout::UNDEFINED,
    };

    let image = unsafe { device.create_image(&image_create_info, None).expect("Failed to create image") };

    let image_memory_requirement = unsafe { device.get_image_memory_requirements(image) };

    let memory_type_index = crate::utility::find_memory_type(
        image_memory_requirement.memory_type_bits,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        device_memory_properties,
    );

    let memory_allocate_info = vk::MemoryAllocateInfo {
        s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
        p_next: std::ptr::null(),
        allocation_size: image_memory_requirement.size,
        memory_type_index,
    };

    let memory = unsafe {
        device
            .allocate_memory(&memory_allocate_info, None)
            .expect("Failed to allocate texture image memory")
    };

    unsafe {
        device.bind_image_memory(image, memory, 0).expect("Failed to bind Image Memmory!");
    }

    (image, memory)
}
