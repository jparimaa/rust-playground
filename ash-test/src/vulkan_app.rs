use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;

#[repr(C)]
#[derive(Clone, Debug, Copy)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 3],
}
const VERTICES_DATA: [Vertex; 3] = [
    Vertex {
        pos: [0.0, -0.5],
        color: [1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [0.5, 0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [-0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    },
];

struct SyncObjects {
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
}

pub struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,
    debug_utils: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    surface: crate::surface::Surface,
    _physical_device: vk::PhysicalDevice,
    device: ash::Device,
    graphics_queue: ash::vk::Queue,
    present_queue: ash::vk::Queue,
    swapchain: crate::swapchain::Swapchain,
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    sync_objects: SyncObjects,
    current_frame_index: usize,
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().expect("Failed to wait device idle");
            for i in 0..self.swapchain.length {
                self.device
                    .destroy_semaphore(self.sync_objects.image_available_semaphores[i], None);
                self.device
                    .destroy_semaphore(self.sync_objects.render_finished_semaphores[i], None);
                self.device.destroy_fence(self.sync_objects.in_flight_fences[i], None);
            }

            self.device.destroy_command_pool(self.command_pool, None);
            for &framebuffer in self.framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            
            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);
            self.device.destroy_render_pass(self.render_pass, None);
            self.swapchain.destroy(&self.device);
            self.device.destroy_device(None);
            self.surface.loader.destroy_surface(self.surface.vk_surface_khr, None);
            self.debug_utils
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.instance.destroy_instance(None);
        }
    }
}

impl VulkanApp {
    pub fn new(window: &winit::window::Window) -> VulkanApp {
        let entry = ash::Entry::new().unwrap();

        if !crate::utility::is_validation_layer_supported(&entry) {
            panic!("Validation layers not supported");
        }

        let instance = crate::instance::create_instance(&entry);
        let (debug_utils, debug_messenger) = crate::instance::create_debug_utils(&entry, &instance);
        let surface = crate::surface::Surface::new(&entry, &instance, window);
        let (physical_device, queue_families) = crate::physical_device::get_physical_device(&instance, &surface);
        let device = crate::device::create_logical_device(&instance, physical_device, &queue_families);
        let graphics_queue = unsafe { device.get_device_queue(queue_families.graphics_family.unwrap(), 0) };
        let present_queue = unsafe { device.get_device_queue(queue_families.present_family.unwrap(), 0) };
        let swapchain = crate::swapchain::Swapchain::new(&instance, &device, physical_device, &surface);
        let render_pass = create_render_pass(&device, swapchain.format);
        let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(&instance, &device, physical_device);
        let framebuffers = create_framebuffers(&device, render_pass, &swapchain.image_views, &swapchain.extent);
        let (pipeline_layout, graphics_pipeline) = create_graphics_pipeline(&device, render_pass, swapchain.extent);
        let command_pool = create_command_pool(&device, &queue_families);
        let command_buffers = create_command_buffers(
            &device,
            command_pool,
            graphics_pipeline,
            &framebuffers,
            render_pass,
            swapchain.extent,
            vertex_buffer,
        );
        let sync_objects = create_sync_objects(&device, swapchain.length);

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
            vertex_buffer,
            vertex_buffer_memory,
            framebuffers,
            pipeline_layout,
            graphics_pipeline,
            command_pool,
            command_buffers: command_buffers,
            sync_objects,
            current_frame_index: 0,
        }
    }

    pub fn draw_frame(&mut self) {
        let frame_index = self.current_frame_index;
        let sync = &self.sync_objects;

        let in_flight_fence = sync.in_flight_fences[frame_index];
        let in_flight_fence_array = [in_flight_fence];

        unsafe {
            self.device
                .wait_for_fences(&in_flight_fence_array, true, std::u64::MAX)
                .expect("Failed to wait for fence");
            self.device
                .reset_fences(&in_flight_fence_array)
                .expect("Failed to reset fence");
        }

        let (image_index, _is_sub_optimal) = unsafe {
            self.swapchain
                .loader
                .acquire_next_image(
                    self.swapchain.vk_swapchain_khr,
                    std::u64::MAX,
                    sync.image_available_semaphores[frame_index],
                    vk::Fence::null(),
                )
                .expect("Failed to acquire next swapchain image")
        };

        let image_available_semaphore = [sync.image_available_semaphores[frame_index]];
        let queue_completed_semaphore = [sync.render_finished_semaphores[frame_index]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: std::ptr::null(),
            wait_semaphore_count: image_available_semaphore.len() as u32,
            p_wait_semaphores: image_available_semaphore.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.command_buffers[image_index as usize],
            signal_semaphore_count: queue_completed_semaphore.len() as u32,
            p_signal_semaphores: queue_completed_semaphore.as_ptr(),
        }];

        unsafe {
            self.device
                .queue_submit(self.graphics_queue, &submit_infos, in_flight_fence)
                .expect("Failed to execute queue submit");
        }

        let swapchains = [self.swapchain.vk_swapchain_khr];

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: std::ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: queue_completed_semaphore.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: std::ptr::null_mut(),
        };

        unsafe {
            self.swapchain
                .loader
                .queue_present(self.present_queue, &present_info)
                .expect("Failed to execute queue present");
        }

        self.current_frame_index = (self.current_frame_index + 1) % self.swapchain.length;
    }
}

fn create_vertex_buffer(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
) -> (vk::Buffer, vk::DeviceMemory) {
    let vertex_buffer_create_info = vk::BufferCreateInfo {
        s_type: vk::StructureType::BUFFER_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::BufferCreateFlags::empty(),
        size: std::mem::size_of_val(&VERTICES_DATA) as u64,
        usage: vk::BufferUsageFlags::VERTEX_BUFFER,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        queue_family_index_count: 0,
        p_queue_family_indices: std::ptr::null(),
    };

    let vertex_buffer = unsafe {
        device
            .create_buffer(&vertex_buffer_create_info, None)
            .expect("Failed to create vertex buffer")
    };

    let mem_requirements = unsafe { device.get_buffer_memory_requirements(vertex_buffer) };
    let mem_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
    let required_memory_flags: vk::MemoryPropertyFlags =
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    let memory_type = find_memory_type(mem_requirements.memory_type_bits, required_memory_flags, mem_properties);

    let allocate_info = vk::MemoryAllocateInfo {
        s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
        p_next: std::ptr::null(),
        allocation_size: mem_requirements.size,
        memory_type_index: memory_type,
    };

    let vertex_buffer_memory = unsafe {
        device
            .allocate_memory(&allocate_info, None)
            .expect("Failed to allocate vertex buffer memory")
    };

    unsafe {
        device
            .bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0)
            .expect("Failed to bind vertex buffer");

        let data_ptr = device
            .map_memory(
                vertex_buffer_memory,
                0,
                vertex_buffer_create_info.size,
                vk::MemoryMapFlags::empty(),
            )
            .expect("Failed to map memory") as *mut Vertex;

        data_ptr.copy_from_nonoverlapping(VERTICES_DATA.as_ptr(), VERTICES_DATA.len());

        device.unmap_memory(vertex_buffer_memory);
    }

    (vertex_buffer, vertex_buffer_memory)
}

fn find_memory_type(
    type_filter: u32,
    required_properties: vk::MemoryPropertyFlags,
    mem_properties: vk::PhysicalDeviceMemoryProperties,
) -> u32 {
    for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
        if (type_filter & (1 << i)) > 0 && memory_type.property_flags.contains(required_properties) {
            return i as u32;
        }
    }

    panic!("Failed to find suitable memory type")
}

fn create_graphics_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
) -> (vk::PipelineLayout, vk::Pipeline) {
    use std::path::Path;
    let vertex_shader_code = crate::utility::read_file(Path::new("ash-test/shader_bin/vert.spv"));
    let fragment_shader_code = crate::utility::read_file(Path::new("ash-test/shader_bin/frag.spv"));

    let vertex_shader_module = crate::utility::create_shader_module(device, vertex_shader_code);
    let fragment_shader_module = crate::utility::create_shader_module(device, fragment_shader_code);

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

    let input_binding = [vk::VertexInputBindingDescription {
        binding: 0,
        stride: std::mem::size_of::<Vertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    }];

    let input_attributes = [
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
    ];

    let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_attribute_description_count: input_attributes.len() as u32,
        p_vertex_attribute_descriptions: input_attributes.as_ptr(),
        vertex_binding_description_count: input_binding.len() as u32,
        p_vertex_binding_descriptions: input_binding.as_ptr(),
    };

    let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
        p_next: std::ptr::null(),
        primitive_restart_enable: vk::FALSE,
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
    };

    let viewports = [vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: swapchain_extent.width as f32,
        height: swapchain_extent.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    }];

    let scissors = [vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: swapchain_extent,
    }];

    let viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        scissor_count: scissors.len() as u32,
        p_scissors: scissors.as_ptr(),
        viewport_count: viewports.len() as u32,
        p_viewports: viewports.as_ptr(),
    };

    let rasterization_statue_create_info = vk::PipelineRasterizationStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineRasterizationStateCreateFlags::empty(),
        depth_clamp_enable: vk::FALSE,
        cull_mode: vk::CullModeFlags::BACK,
        front_face: vk::FrontFace::CLOCKWISE,
        line_width: 1.0,
        polygon_mode: vk::PolygonMode::FILL,
        rasterizer_discard_enable: vk::FALSE,
        depth_bias_clamp: 0.0,
        depth_bias_constant_factor: 0.0,
        depth_bias_enable: vk::FALSE,
        depth_bias_slope_factor: 0.0,
    };

    let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        flags: vk::PipelineMultisampleStateCreateFlags::empty(),
        p_next: std::ptr::null(),
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        sample_shading_enable: vk::FALSE,
        min_sample_shading: 0.0,
        p_sample_mask: std::ptr::null(),
        alpha_to_one_enable: vk::FALSE,
        alpha_to_coverage_enable: vk::FALSE,
    };

    let stencil_state = vk::StencilOpState {
        fail_op: vk::StencilOp::KEEP,
        pass_op: vk::StencilOp::KEEP,
        depth_fail_op: vk::StencilOp::KEEP,
        compare_op: vk::CompareOp::ALWAYS,
        compare_mask: 0,
        write_mask: 0,
        reference: 0,
    };

    let depth_state_create_info = vk::PipelineDepthStencilStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
        depth_test_enable: vk::FALSE,
        depth_write_enable: vk::FALSE,
        depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
        depth_bounds_test_enable: vk::FALSE,
        stencil_test_enable: vk::FALSE,
        front: stencil_state,
        back: stencil_state,
        max_depth_bounds: 1.0,
        min_depth_bounds: 0.0,
    };

    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
        blend_enable: vk::FALSE,
        color_write_mask: vk::ColorComponentFlags::all(),
        src_color_blend_factor: vk::BlendFactor::ONE,
        dst_color_blend_factor: vk::BlendFactor::ZERO,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ONE,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
    }];

    let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: vk::FALSE,
        logic_op: vk::LogicOp::COPY,
        attachment_count: color_blend_attachment_states.len() as u32,
        p_attachments: color_blend_attachment_states.as_ptr(),
        blend_constants: [0.0, 0.0, 0.0, 0.0],
    };

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 0,
        p_set_layouts: std::ptr::null(),
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    };

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

fn create_command_pool(
    device: &ash::Device,
    queue_family_indices: &crate::utility::QueueFamilyIndices,
) -> vk::CommandPool {
    let command_pool_create_info = vk::CommandPoolCreateInfo {
        s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::CommandPoolCreateFlags::empty(),
        queue_family_index: queue_family_indices.graphics_family.unwrap(),
    };

    unsafe {
        device
            .create_command_pool(&command_pool_create_info, None)
            .expect("Failed to create command pool")
    }
}

fn create_command_buffers(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    graphics_pipeline: vk::Pipeline,
    framebuffers: &Vec<vk::Framebuffer>,
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    vertex_buffer: vk::Buffer,
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

        unsafe {
            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin command buffer");
            device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline);            
            let vertex_buffers = [vertex_buffer];
            let offsets = [0_u64];
            device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            device.cmd_draw(command_buffer, VERTICES_DATA.len() as u32, 1, 0, 0);
            device.cmd_end_render_pass(command_buffer);
            device
                .end_command_buffer(command_buffer)
                .expect("Failed to end command buffer");
        }
    }

    command_buffers
}

fn create_sync_objects(device: &ash::Device, num_objects: usize) -> SyncObjects {
    let mut sync_objects = SyncObjects {
        image_available_semaphores: vec![],
        render_finished_semaphores: vec![],
        in_flight_fences: vec![],
    };

    let semaphore_create_info = vk::SemaphoreCreateInfo {
        s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::SemaphoreCreateFlags::empty(),
    };

    let fence_create_info = vk::FenceCreateInfo {
        s_type: vk::StructureType::FENCE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::FenceCreateFlags::SIGNALED,
    };

    for _ in 0..num_objects {
        unsafe {
            let image_available_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .expect("Failed to create semaphore");
            let render_finished_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .expect("Failed to create semaphore");
            let in_flight_fence = device
                .create_fence(&fence_create_info, None)
                .expect("Failed to create fence");

            sync_objects.image_available_semaphores.push(image_available_semaphore);
            sync_objects.render_finished_semaphores.push(render_finished_semaphore);
            sync_objects.in_flight_fences.push(in_flight_fence);
        }
    }

    sync_objects
}
