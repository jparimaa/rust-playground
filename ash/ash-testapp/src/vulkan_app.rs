use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;

#[repr(C)]
#[derive(Clone, Debug, Copy)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 3],
    tex_coord: [f32; 2],
}

const VERTICES_DATA: [Vertex; 4] = [
    Vertex {
        pos: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        pos: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        pos: [0.5, 0.5],
        color: [0.0, 0.0, 1.0],
        tex_coord: [0.0, 1.0],
    },
    Vertex {
        pos: [-0.5, 0.5],
        color: [0.0, 1.0, 0.0],
        tex_coord: [1.0, 1.0],
    },
];

const INDICES_DATA: [u32; 6] = [0, 1, 2, 2, 3, 0];

#[repr(C)]
#[derive(Clone, Debug, Copy)]
struct WVPMatrices {
    world: cgmath::Matrix4<f32>,
    view: cgmath::Matrix4<f32>,
    projection: cgmath::Matrix4<f32>,
}

pub struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,

    debug_utils: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,

    surface: util::surface::Surface,
    _physical_device: vk::PhysicalDevice,
    device: ash::Device,
    graphics_queue: ash::vk::Queue,
    present_queue: ash::vk::Queue,

    swapchain: util::swapchain::Swapchain,
    render_pass: vk::RenderPass,

    transform_desc_set_layout: vk::DescriptorSetLayout,
    texture_desc_set_layout: vk::DescriptorSetLayout,
    framebuffers: Vec<vk::Framebuffer>,

    texture: util::texture::Texture,
    sampler: vk::Sampler,

    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,

    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,

    ubo_data: WVPMatrices,
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,

    descriptor_pool: vk::DescriptorPool,
    _transform_desc_sets: Vec<vk::DescriptorSet>,

    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,

    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    presenter: util::presenter::Presenter,
    time_instant: std::time::Instant,
    total_duration: std::time::Duration,
}

impl VulkanApp {
    pub fn new(window: &winit::window::Window) -> VulkanApp {
        let entry = ash::Entry::new().unwrap();

        if !util::common::is_validation_layer_supported(&entry) {
            panic!("Validation layers not supported");
        }

        let instance = util::instance::create_instance(&entry);
        let (debug_utils, debug_messenger) = util::instance::create_debug_utils(&entry, &instance);
        let surface = util::surface::Surface::new(&entry, &instance, window);
        let (physical_device, queue_families) = util::physical_device::get_physical_device(&instance, &surface);
        let memory_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let device = util::device::create_logical_device(&instance, physical_device, &queue_families);
        let graphics_queue = unsafe { device.get_device_queue(queue_families.graphics_family.unwrap(), 0) };
        let present_queue = unsafe { device.get_device_queue(queue_families.present_family.unwrap(), 0) };
        let swapchain = util::swapchain::Swapchain::new(&instance, &device, physical_device, &surface, crate::constants::WINDOW_WIDTH, crate::constants::WINDOW_HEIGHT);
        //
        let render_pass = create_render_pass(&device, swapchain.format);
        let command_pool = util::common::create_command_pool(&device, queue_families.graphics_family.unwrap());
        //
        let mut texture = util::texture::Texture::new(
            &device,
            command_pool,
            graphics_queue,
            &memory_properties,
            &std::path::Path::new("assets/checker.png"),
        );
        let _image_view = texture.get_or_create_image_view(vk::Format::R8G8B8A8_UNORM);
        let sampler = create_sampler(&device);
        //
        let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(&device, &memory_properties, command_pool, graphics_queue);
        let (index_buffer, index_buffer_memory) = create_index_buffer(&device, &memory_properties, command_pool, graphics_queue);
        let (uniform_buffers, uniform_buffers_memory) = create_uniform_buffers(&device, &memory_properties, swapchain.length);
        //
        let descriptor_pool = create_descriptor_pool(&device, swapchain.length + 1);
        let transform_desc_set_layout = create_transform_desc_set_layout(&device);
        let texture_desc_set_layout = create_texture_desc_set_layout(&device);
        let transform_desc_sets = create_transform_desc_sets(
            &device,
            descriptor_pool,
            transform_desc_set_layout,
            swapchain.length,
            &uniform_buffers,
        );
        let texture_desc_set = create_texture_desc_set(&device, descriptor_pool, texture_desc_set_layout, sampler, &mut texture);
        //
        let framebuffers = create_framebuffers(&device, render_pass, &swapchain.image_views, &swapchain.extent);

        let desc_set_layouts = vec![transform_desc_set_layout, texture_desc_set_layout];
        let (pipeline_layout, graphics_pipeline) = create_graphics_pipeline(&device, render_pass, swapchain.extent, &desc_set_layouts);
        let command_buffers = create_command_buffers(
            &device,
            command_pool,
            graphics_pipeline,
            &framebuffers,
            render_pass,
            swapchain.extent,
            vertex_buffer,
            index_buffer,
            pipeline_layout,
            &transform_desc_sets,
            texture_desc_set
        );
        let presenter = util::presenter::Presenter::new(&device, swapchain.length);

        use cgmath::SquareMatrix;

        let matrices = WVPMatrices {
            world: cgmath::Matrix4::<f32>::identity(),
            view: cgmath::Matrix4::look_at(
                cgmath::Point3::new(0.0, 0.0, 2.0),  // eye
                cgmath::Point3::new(0.0, 0.0, 0.0),  // point
                cgmath::Vector3::new(0.0, 1.0, 0.0), // up
            ),
            projection: cgmath::perspective(
                cgmath::Deg(45.0),
                swapchain.extent.width as f32 / swapchain.extent.height as f32,
                0.1,
                100.0,
            ),
        };

        VulkanApp {
            _entry: entry,
            instance,
            debug_utils,
            debug_messenger,
            surface,
            _physical_device: physical_device,
            device,
            graphics_queue,
            present_queue,
            swapchain,
            render_pass,
            transform_desc_set_layout,
            texture_desc_set_layout,
            texture,
            sampler,
            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,
            ubo_data: matrices,
            uniform_buffers,
            uniform_buffers_memory,
            descriptor_pool,
            _transform_desc_sets: transform_desc_sets,
            framebuffers,
            pipeline_layout,
            graphics_pipeline,
            command_pool,
            command_buffers,
            presenter,
            time_instant: std::time::Instant::now(),
            total_duration: std::time::Duration::new(0, 0),
        }
    }

    pub fn draw(&mut self) {
        let time_delta = (self.time_instant.elapsed().as_millis() as f32 - self.total_duration.as_millis() as f32) / 1000.0;
        self.total_duration = self.time_instant.elapsed();

        let image_index = self.presenter.acquire_image(&self.swapchain);

        self.ubo_data.world =
            cgmath::Matrix4::from_axis_angle(cgmath::Vector3::new(0.0, 0.0, 1.0), cgmath::Deg(90.0) * time_delta) * self.ubo_data.world;
        // Todo: avoid copy
        let ubos = [self.ubo_data.clone()];
        let buffer_size = (std::mem::size_of::<WVPMatrices>() * ubos.len()) as u64;

        unsafe {
            let data_ptr = self
                .device
                .map_memory(
                    self.uniform_buffers_memory[image_index as usize],
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to map memory") as *mut WVPMatrices;
            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            self.device.unmap_memory(self.uniform_buffers_memory[image_index as usize]);
        }

        self.presenter.present(
            &self.device,
            &self.swapchain,
            &self.command_buffers,
            self.graphics_queue,
            self.present_queue,
            image_index,
        );
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().expect("Failed to wait device idle");
            self.presenter.destroy(&self.device);

            self.device.destroy_command_pool(self.command_pool, None);
            for &framebuffer in self.framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_buffer(self.index_buffer, None);

            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            for i in 0..self.uniform_buffers.len() {
                self.device.destroy_buffer(self.uniform_buffers[i], None);
                self.device.free_memory(self.uniform_buffers_memory[i], None);
            }
            self.device.free_memory(self.index_buffer_memory, None);
            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);
            self.device.destroy_sampler(self.sampler, None);
            self.texture.destroy(&self.device);
            self.device.destroy_descriptor_set_layout(self.texture_desc_set_layout, None);
            self.device.destroy_descriptor_set_layout(self.transform_desc_set_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            self.swapchain.destroy(&self.device);
            self.device.destroy_device(None);
            self.surface.loader.destroy_surface(self.surface.vk_surface_khr, None);
            self.debug_utils.destroy_debug_utils_messenger(self.debug_messenger, None);
            self.instance.destroy_instance(None);
        }
    }
}

fn create_render_pass(device: &ash::Device, surface_format: vk::Format) -> vk::RenderPass {
    let color_attachment = vk::AttachmentDescription {
        flags: vk::AttachmentDescriptionFlags::empty(),
        format: surface_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
    };

    let color_attachment_ref = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    };

    let subpasses = [vk::SubpassDescription {
        color_attachment_count: 1,
        p_color_attachments: &color_attachment_ref,
        p_depth_stencil_attachment: std::ptr::null(),
        flags: vk::SubpassDescriptionFlags::empty(),
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        input_attachment_count: 0,
        p_input_attachments: std::ptr::null(),
        p_resolve_attachments: std::ptr::null(),
        preserve_attachment_count: 0,
        p_preserve_attachments: std::ptr::null(),
    }];

    let render_pass_attachments = [color_attachment];

    let subpass_dependencies = [vk::SubpassDependency {
        src_subpass: vk::SUBPASS_EXTERNAL,
        dst_subpass: 0,
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        src_access_mask: vk::AccessFlags::empty(),
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        dependency_flags: vk::DependencyFlags::empty(),
    }];

    let renderpass_create_info = vk::RenderPassCreateInfo {
        s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
        flags: vk::RenderPassCreateFlags::empty(),
        p_next: std::ptr::null(),
        attachment_count: render_pass_attachments.len() as u32,
        p_attachments: render_pass_attachments.as_ptr(),
        subpass_count: subpasses.len() as u32,
        p_subpasses: subpasses.as_ptr(),
        dependency_count: subpass_dependencies.len() as u32,
        p_dependencies: subpass_dependencies.as_ptr(),
    };

    let render_pass = unsafe {
        device
            .create_render_pass(&renderpass_create_info, None)
            .expect("Failed to create render pass!")
    };

    render_pass
}

fn create_sampler(device: &ash::Device) -> vk::Sampler {
    let sampler_create_info = vk::SamplerCreateInfo {
        s_type: vk::StructureType::SAMPLER_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::SamplerCreateFlags::empty(),
        mag_filter: vk::Filter::LINEAR,
        min_filter: vk::Filter::LINEAR,
        mipmap_mode: vk::SamplerMipmapMode::LINEAR,
        address_mode_u: vk::SamplerAddressMode::REPEAT,
        address_mode_v: vk::SamplerAddressMode::REPEAT,
        address_mode_w: vk::SamplerAddressMode::REPEAT,
        mip_lod_bias: 0.0,
        anisotropy_enable: vk::FALSE,
        max_anisotropy: 16.0,
        compare_enable: vk::FALSE,
        compare_op: vk::CompareOp::ALWAYS,
        min_lod: 0.0,
        max_lod: 0.0,
        border_color: vk::BorderColor::INT_OPAQUE_BLACK,
        unnormalized_coordinates: vk::FALSE,
    };

    unsafe { device.create_sampler(&sampler_create_info, None).expect("Failed to create sampler") }
}

fn create_vertex_buffer(
    device: &ash::Device,
    physical_device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    command_pool: vk::CommandPool,
    submit_queue: vk::Queue,
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_size = std::mem::size_of_val(&VERTICES_DATA) as vk::DeviceSize;

    let (staging_buffer, staging_buffer_memory) = util::common::create_buffer(
        device,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        &physical_device_memory_properties,
    );

    unsafe {
        let data_ptr = device
            .map_memory(staging_buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
            .expect("Failed to map memory") as *mut Vertex;

        data_ptr.copy_from_nonoverlapping(VERTICES_DATA.as_ptr(), VERTICES_DATA.len());

        device.unmap_memory(staging_buffer_memory);
    }

    let (vertex_buffer, vertex_buffer_memory) = util::common::create_buffer(
        device,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        &physical_device_memory_properties,
    );

    util::common::copy_buffer(device, submit_queue, command_pool, staging_buffer, vertex_buffer, buffer_size);

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }

    (vertex_buffer, vertex_buffer_memory)
}

fn create_index_buffer(
    device: &ash::Device,
    physical_device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    command_pool: vk::CommandPool,
    submit_queue: vk::Queue,
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_size = std::mem::size_of_val(&INDICES_DATA) as vk::DeviceSize;

    let (staging_buffer, staging_buffer_memory) = util::common::create_buffer(
        device,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        &physical_device_memory_properties,
    );

    unsafe {
        let data_ptr = device
            .map_memory(staging_buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
            .expect("Failed to map memory") as *mut u32;

        data_ptr.copy_from_nonoverlapping(INDICES_DATA.as_ptr(), INDICES_DATA.len());

        device.unmap_memory(staging_buffer_memory);
    }

    let (index_buffer, index_buffer_memory) = util::common::create_buffer(
        device,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        &physical_device_memory_properties,
    );

    util::common::copy_buffer(device, submit_queue, command_pool, staging_buffer, index_buffer, buffer_size);

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }

    (index_buffer, index_buffer_memory)
}

fn create_uniform_buffers(
    device: &ash::Device,
    physical_device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    num_buffers: usize,
) -> (Vec<vk::Buffer>, Vec<vk::DeviceMemory>) {
    let buffer_size = std::mem::size_of::<WVPMatrices>();

    let mut uniform_buffers = vec![];
    let mut uniform_buffers_memory = vec![];

    for _ in 0..num_buffers {
        let (uniform_buffer, uniform_buffer_memory) = util::common::create_buffer(
            device,
            buffer_size as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            physical_device_memory_properties,
        );
        uniform_buffers.push(uniform_buffer);
        uniform_buffers_memory.push(uniform_buffer_memory);
    }

    (uniform_buffers, uniform_buffers_memory)
}

fn create_descriptor_pool(device: &ash::Device, num_max_sets: usize) -> vk::DescriptorPool {
    let pool_sizes = [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: num_max_sets as u32,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
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

fn create_transform_desc_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
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

fn create_texture_desc_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
    let bindings = [vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::FRAGMENT,
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

fn create_transform_desc_sets(
    device: &ash::Device,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    desc_set_count: usize,
    uniforms_buffers: &Vec<vk::Buffer>,
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
            buffer: uniforms_buffers[i],
            offset: 0,
            range: std::mem::size_of::<WVPMatrices>() as u64,
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

fn create_texture_desc_set(
    device: &ash::Device,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    sampler: vk::Sampler,
    texture: &mut util::texture::Texture,
) -> vk::DescriptorSet {
    let layouts: Vec<vk::DescriptorSetLayout> = vec![descriptor_set_layout];

    let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
        p_next: std::ptr::null(),
        descriptor_pool,
        descriptor_set_count: layouts.len() as u32,
        p_set_layouts: layouts.as_ptr(),
    };

    let desc_set = unsafe {
        device
            .allocate_descriptor_sets(&descriptor_set_allocate_info)
            .expect("Failed to allocate descriptor sets!")[0]
    };

    let descriptor_image_info = [vk::DescriptorImageInfo {
        sampler: sampler,
        image_view: texture.get_or_create_image_view(vk::Format::R8G8B8A8_UNORM),
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    }];

    let descriptor_write_sets = [vk::WriteDescriptorSet {
        s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
        p_next: std::ptr::null(),
        dst_set: desc_set,
        dst_binding: 0,
        dst_array_element: 0,
        descriptor_count: 1,
        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        p_buffer_info: std::ptr::null(),
        p_image_info: descriptor_image_info.as_ptr(),
        p_texel_buffer_view: std::ptr::null(),
    }];

    unsafe {
        device.update_descriptor_sets(&descriptor_write_sets, &[]);
    }

    desc_set
}

fn create_framebuffers(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    image_views: &Vec<vk::ImageView>,
    swapchain_extent: &vk::Extent2D,
) -> Vec<vk::Framebuffer> {
    let mut framebuffers = vec![];

    for &image_view in image_views.iter() {
        let attachments = [image_view];

        let framebuffer_create_info = vk::FramebufferCreateInfo {
            s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::FramebufferCreateFlags::empty(),
            render_pass,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            width: swapchain_extent.width,
            height: swapchain_extent.height,
            layers: 1,
        };

        let framebuffer = unsafe {
            device
                .create_framebuffer(&framebuffer_create_info, None)
                .expect("Failed to create framebuffer")
        };

        framebuffers.push(framebuffer);
    }

    framebuffers
}

fn create_graphics_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    desc_set_layouts: &Vec<vk::DescriptorSetLayout>,
) -> (vk::PipelineLayout, vk::Pipeline) {
    use std::path::Path;
    let vertex_shader_code = util::common::read_file(Path::new("ash/ash-testapp/shader_bin/vert.spv"));
    let fragment_shader_code = util::common::read_file(Path::new("ash/ash-testapp/shader_bin/frag.spv"));

    let vertex_shader_module = util::common::create_shader_module(device, vertex_shader_code);
    let fragment_shader_module = util::common::create_shader_module(device, fragment_shader_code);

    let main_function_name = std::ffi::CString::new("main").unwrap();

    let shader_stages = [
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            module: vertex_shader_module,
            p_name: main_function_name.as_ptr(),
            p_specialization_info: std::ptr::null(),
            stage: vk::ShaderStageFlags::VERTEX,
        },
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            module: fragment_shader_module,
            p_name: main_function_name.as_ptr(),
            p_specialization_info: std::ptr::null(),
            stage: vk::ShaderStageFlags::FRAGMENT,
        },
    ];

    let input_binding = vec![vk::VertexInputBindingDescription {
        binding: 0,
        stride: std::mem::size_of::<Vertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    }];

    let input_attributes = vec![
        vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: memoffset::offset_of!(Vertex, pos) as u32,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: memoffset::offset_of!(Vertex, color) as u32,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 2,
            format: vk::Format::R32G32_SFLOAT,
            offset: memoffset::offset_of!(Vertex, tex_coord) as u32,
        },
    ];

    use util::pipeline;

    let vertex_input_state_create_info = pipeline::get_default_vertex_input_state(&input_attributes, &input_binding);
    let vertex_input_assembly_state_info = pipeline::get_default_input_assembly_state();
    let viewports = pipeline::get_default_viewports(swapchain_extent);
    let scissors = pipeline::get_default_scissors(swapchain_extent);
    let viewport_state_create_info = pipeline::get_default_viewport_state(&viewports, &scissors);
    let rasterization_statue_create_info = pipeline::get_default_rasterization_state();
    let multisample_state_create_info = pipeline::get_default_multisample_state();
    let depth_state_create_info = pipeline::get_default_depth_stencil_state();
    let color_blend_attachments = pipeline::get_default_color_blend_attachments();
    let color_blend_state = pipeline::get_default_color_blend_state(&color_blend_attachments);
    let pipeline_layout_create_info = pipeline::get_default_pipeline_layout(&desc_set_layouts);

    let pipeline_layout = unsafe {
        device
            .create_pipeline_layout(&pipeline_layout_create_info, None)
            .expect("Failed to create pipeline layout")
    };

    let graphic_pipeline_create_infos = [vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: shader_stages.len() as u32,
        p_stages: shader_stages.as_ptr(),
        p_vertex_input_state: &vertex_input_state_create_info,
        p_input_assembly_state: &vertex_input_assembly_state_info,
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: &viewport_state_create_info,
        p_rasterization_state: &rasterization_statue_create_info,
        p_multisample_state: &multisample_state_create_info,
        p_depth_stencil_state: &depth_state_create_info,
        p_color_blend_state: &color_blend_state,
        p_dynamic_state: std::ptr::null(),
        layout: pipeline_layout,
        render_pass,
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: -1,
    }];

    let graphics_pipelines = unsafe {
        device
            .create_graphics_pipelines(vk::PipelineCache::null(), &graphic_pipeline_create_infos, None)
            .expect("Failed to create graphics pipeline")
    };

    unsafe {
        device.destroy_shader_module(vertex_shader_module, None);
        device.destroy_shader_module(fragment_shader_module, None);
    }

    (pipeline_layout, graphics_pipelines[0])
}

fn create_command_buffers(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    graphics_pipeline: vk::Pipeline,
    framebuffers: &Vec<vk::Framebuffer>,
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    pipeline_layout: vk::PipelineLayout,
    transform_desc_sets: &Vec<vk::DescriptorSet>,
    texture_desc_set: vk::DescriptorSet
) -> Vec<vk::CommandBuffer> {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: std::ptr::null(),
        command_buffer_count: framebuffers.len() as u32,
        command_pool,
        level: vk::CommandBufferLevel::PRIMARY,
    };

    let command_buffers = unsafe {
        device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate Command Buffers!")
    };

    for (i, &command_buffer) in command_buffers.iter().enumerate() {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: std::ptr::null(),
            p_inheritance_info: std::ptr::null(),
            flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        };

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.2, 1.0],
            },
        }];

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: std::ptr::null(),
            render_pass,
            framebuffer: framebuffers[i],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain_extent,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
        };

        let transform_desc_set = [transform_desc_sets[i], texture_desc_set];

        unsafe {
            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin command buffer");
            device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline);
            let vertex_buffers = [vertex_buffer];
            let offsets = [0_u64];
            device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                0,
                &transform_desc_set,
                &[],
            );
            device.cmd_draw_indexed(command_buffer, INDICES_DATA.len() as u32, 1, 0, 0, 0);
            device.cmd_end_render_pass(command_buffer);
            device.end_command_buffer(command_buffer).expect("Failed to end command buffer");
        }
    }

    command_buffers
}
