use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_descriptor_pool(device: &ash::Device, num_max_sets: usize) -> vk::DescriptorPool {
    let pool_sizes = [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: num_max_sets as u32,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 3,
        },
    ];

    let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: num_max_sets as u32,
        pool_size_count: pool_sizes.len() as u32,
        p_pool_sizes: pool_sizes.as_ptr(),
    };

    unsafe {
        device
            .create_descriptor_pool(&descriptor_pool_create_info, None)
            .expect("Failed to create descriptor pool")
    }
}

pub fn create_transform_desc_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
    let bindings = [vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: std::ptr::null(),
    }];

    let desc_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: bindings.len() as u32,
        p_bindings: bindings.as_ptr(),
    };

    unsafe {
        device
            .create_descriptor_set_layout(&desc_set_layout_create_info, None)
            .expect("Failed to create descriptor set layout")
    }
}

pub fn create_texture_desc_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
    let bindings = [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 2,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
    ];

    let desc_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: bindings.len() as u32,
        p_bindings: bindings.as_ptr(),
    };

    unsafe {
        device
            .create_descriptor_set_layout(&desc_set_layout_create_info, None)
            .expect("Failed to create descriptor set layout")
    }
}

pub fn create_transform_desc_sets(
    device: &ash::Device,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    desc_set_count: usize,
    uniforms_buffers: &Vec<crate::buffer::Buffer>,
) -> Vec<vk::DescriptorSet> {
    let mut layouts: Vec<vk::DescriptorSetLayout> = vec![];
    for _ in 0..desc_set_count {
        layouts.push(descriptor_set_layout);
    }

    let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
        p_next: std::ptr::null(),
        descriptor_pool,
        descriptor_set_count: desc_set_count as u32,
        p_set_layouts: layouts.as_ptr(),
    };

    let desc_sets = unsafe {
        device
            .allocate_descriptor_sets(&descriptor_set_allocate_info)
            .expect("Failed to allocate descriptor sets!")
    };

    for (i, &desc_set) in desc_sets.iter().enumerate() {
        let desc_buffer_info = [vk::DescriptorBufferInfo {
            buffer: uniforms_buffers[i].buffer,
            offset: 0,
            range: std::mem::size_of::<crate::data::WVPMatrices>() as u64,
        }];

        let descriptor_write_sets = [vk::WriteDescriptorSet {
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            p_next: std::ptr::null(),
            dst_set: desc_set,
            dst_binding: 0,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            p_image_info: std::ptr::null(),
            p_buffer_info: desc_buffer_info.as_ptr(),
            p_texel_buffer_view: std::ptr::null(),
        }];

        unsafe {
            device.update_descriptor_sets(&descriptor_write_sets, &[]);
        }
    }

    desc_sets
}

pub fn create_texture_desc_set(
    device: &ash::Device,
    desc_pool: vk::DescriptorPool,
    desc_set_layout: vk::DescriptorSetLayout,
    sampler: vk::Sampler,
    textures: &mut std::vec::Vec<&mut util::image::Image>,
) -> vk::DescriptorSet {
    let layouts: Vec<vk::DescriptorSetLayout> = vec![desc_set_layout];

    let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
        p_next: std::ptr::null(),
        descriptor_pool: desc_pool,
        descriptor_set_count: layouts.len() as u32,
        p_set_layouts: layouts.as_ptr(),
    };

    let desc_set = unsafe {
        device
            .allocate_descriptor_sets(&descriptor_set_allocate_info)
            .expect("Failed to allocate descriptor sets!")[0]
    };

    let mut desc_image_infos = vec![];
    for t in textures {
        desc_image_infos.push(vk::DescriptorImageInfo {
            sampler: sampler,
            image_view: t.get_or_create_image_view(vk::Format::R8G8B8A8_UNORM, vk::ImageAspectFlags::COLOR),
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        });
    }

    let mut desc_write_sets = vec![];
    for (i, info) in desc_image_infos.iter().enumerate() {
        desc_write_sets.push(vk::WriteDescriptorSet {
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            p_next: std::ptr::null(),
            dst_set: desc_set,
            dst_binding: i as u32,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_buffer_info: std::ptr::null(),
            p_image_info: info,
            p_texel_buffer_view: std::ptr::null(),
        });
    }

    unsafe {
        device.update_descriptor_sets(&desc_write_sets, &[]);
    }

    desc_set
}
