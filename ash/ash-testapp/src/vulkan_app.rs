use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;

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
    color_buffer: util::image::Image,
    depth_buffer: util::image::Image,
    framebuffers: Vec<vk::Framebuffer>,

    texture: util::image::Image,
    sampler: vk::Sampler,

    vertex_buffer: crate::buffer::Buffer,
    index_buffer: crate::buffer::Buffer,
    uniform_buffers: Vec<crate::buffer::Buffer>,
    ubo_data: crate::data::WVPMatrices,

    descriptor_pool: vk::DescriptorPool,
    _transform_desc_sets: Vec<vk::DescriptorSet>,

    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,

    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    presenter: util::presenter::Presenter,
    time_instant: std::time::Instant,
    total_duration: std::time::Duration,

    camera: util::camera::Camera,
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
        let swapchain = util::swapchain::Swapchain::new(
            &instance,
            &device,
            physical_device,
            &surface,
            crate::constants::WINDOW_WIDTH,
            crate::constants::WINDOW_HEIGHT,
        );
        //
        let msaa_samples = vk::SampleCountFlags::TYPE_8;
        let render_pass = crate::pipeline::create_render_pass(&device, swapchain.format, msaa_samples);
        let mut color_buffer = util::image::Image::color_target(
            &device,
            swapchain.format,
            swapchain.extent.width,
            swapchain.extent.height,
            msaa_samples,
            &memory_properties,
        );
        let color_view = color_buffer.get_or_create_image_view(swapchain.format, vk::ImageAspectFlags::COLOR);
        let mut depth_buffer = util::image::Image::depth_target(
            &device,
            crate::constants::DEPTH_FORMAT,
            swapchain.extent.width,
            swapchain.extent.height,
            msaa_samples,
            &memory_properties,
        );
        let depth_view = depth_buffer.get_or_create_image_view(crate::constants::DEPTH_FORMAT, vk::ImageAspectFlags::DEPTH);
        let framebuffers = crate::pipeline::create_framebuffers(
            &device,
            render_pass,
            &swapchain.image_views,
            &color_view,
            &depth_view,
            &swapchain.extent,
        );
        //
        let command_pool = util::command::create_command_pool(&device, queue_families.graphics_family.unwrap());
        let graphics_queue = unsafe { device.get_device_queue(queue_families.graphics_family.unwrap(), 0) };
        let present_queue = unsafe { device.get_device_queue(queue_families.present_family.unwrap(), 0) };
        //
        let model = util::gltf_model::GltfModel::new(&std::path::Path::new("C:/Projects/rust-playground/assets/DamagedHelmet.gltf"));
        let mut texture = util::image::Image::from_texture(
            &device,
            command_pool,
            graphics_queue,
            &memory_properties,
            &model.materials[0].base_texture,
        );
        let _image_view = texture.get_or_create_image_view(vk::Format::R8G8B8A8_UNORM, vk::ImageAspectFlags::COLOR);
        let sampler = crate::sampler::create_sampler(&device);
        //
        use crate::buffer;
        let vertex_buffer = buffer::create_vertex_buffer(&device, &memory_properties, command_pool, graphics_queue, &model.meshes[0]);
        let index_buffer = buffer::create_index_buffer(&device, &memory_properties, command_pool, graphics_queue, &model.meshes[0]);
        let uniform_buffers = buffer::create_uniform_buffers(&device, &memory_properties, swapchain.length);
        //
        use crate::desc_set;
        let descriptor_pool = desc_set::create_descriptor_pool(&device, swapchain.length + 1);
        let transform_desc_set_layout = desc_set::create_transform_desc_set_layout(&device);
        let texture_desc_set_layout = desc_set::create_texture_desc_set_layout(&device);
        let transform_desc_sets = desc_set::create_transform_desc_sets(
            &device,
            descriptor_pool,
            transform_desc_set_layout,
            swapchain.length,
            &uniform_buffers,
        );
        let texture_desc_set = desc_set::create_texture_desc_set(&device, descriptor_pool, texture_desc_set_layout, sampler, &mut texture);
        //
        let desc_set_layouts = vec![transform_desc_set_layout, texture_desc_set_layout];
        let (pipeline_layout, graphics_pipeline) =
            crate::pipeline::create_graphics_pipeline(&device, render_pass, swapchain.extent, &desc_set_layouts, msaa_samples);
        //
        let command_buffers = create_command_buffers(
            &device,
            command_pool,
            graphics_pipeline,
            &framebuffers,
            render_pass,
            swapchain.extent,
            &vertex_buffer,
            &index_buffer,
            model.meshes[0].indices.len() as u32,
            pipeline_layout,
            &transform_desc_sets,
            texture_desc_set,
        );
        let presenter = util::presenter::Presenter::new(&device, swapchain.length);

        use cgmath::SquareMatrix;

        let matrices = crate::data::WVPMatrices {
            world: cgmath::Matrix4::<f32>::identity(),
            view: cgmath::Matrix4::<f32>::identity(),
            projection: cgmath::Matrix4::<f32>::identity(),
        };

        let camera = util::camera::Camera::new(0.3, 1.0, (crate::constants::WINDOW_WIDTH / crate::constants::WINDOW_HEIGHT) as f32);

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
            index_buffer,
            ubo_data: matrices,
            uniform_buffers,
            descriptor_pool,
            _transform_desc_sets: transform_desc_sets,
            color_buffer,
            depth_buffer,
            framebuffers,
            pipeline_layout,
            graphics_pipeline,
            command_pool,
            command_buffers,
            presenter,
            time_instant: std::time::Instant::now(),
            total_duration: std::time::Duration::new(0, 0),
            camera,
        }
    }

    pub fn draw(&mut self) {
        let time_delta = (self.time_instant.elapsed().as_millis() as f32 - self.total_duration.as_millis() as f32) / 1000.0;
        self.total_duration = self.time_instant.elapsed();

        self.camera.update(time_delta);

        let image_index = self.presenter.acquire_image(&self.swapchain);

        self.ubo_data.world =
            cgmath::Matrix4::from_axis_angle(cgmath::Vector3::new(-0.3, 1.5, 0.5), cgmath::Deg(10.0) * time_delta) * self.ubo_data.world;
        self.ubo_data.view = self.camera.get_view_matrix();
        self.ubo_data.projection = self.camera.get_projection_matrix();
        // Todo: avoid copy
        let ubos = [self.ubo_data.clone()];
        let buffer_size = (std::mem::size_of::<crate::data::WVPMatrices>() * ubos.len()) as u64;

        unsafe {
            let data_ptr = self
                .device
                .map_memory(
                    self.uniform_buffers[image_index as usize].memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to map memory") as *mut crate::data::WVPMatrices;
            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            self.device.unmap_memory(self.uniform_buffers[image_index as usize].memory);
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

    pub fn handle_key_input(&mut self, key: winit::event::VirtualKeyCode, state: winit::event::ElementState) {
        self.camera.handle_key_input(key, state);
    }

    pub fn handle_mouse_movement(&mut self, x: i32, y: i32) {
        self.camera.handle_mouse_movement(x, y);
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
            self.depth_buffer.destroy(&self.device);
            self.color_buffer.destroy(&self.device);
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            for i in 0..self.uniform_buffers.len() {
                self.uniform_buffers[i].destroy();
            }
            self.index_buffer.destroy();
            self.vertex_buffer.destroy();
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

fn create_command_buffers(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    graphics_pipeline: vk::Pipeline,
    framebuffers: &Vec<vk::Framebuffer>,
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    vertex_buffer: &crate::buffer::Buffer,
    index_buffer: &crate::buffer::Buffer,
    num_indices: u32,
    pipeline_layout: vk::PipelineLayout,
    transform_desc_sets: &Vec<vk::DescriptorSet>,
    texture_desc_set: vk::DescriptorSet,
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

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.2, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
            },
        ];

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
            let vertex_buffers = [vertex_buffer.buffer];
            let offsets = [0_u64];
            device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            device.cmd_bind_index_buffer(command_buffer, index_buffer.buffer, 0, vk::IndexType::UINT32);
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                0,
                &transform_desc_set,
                &[],
            );
            device.cmd_draw_indexed(command_buffer, num_indices, 1, 0, 0, 0);
            device.cmd_end_render_pass(command_buffer);
            device.end_command_buffer(command_buffer).expect("Failed to end command buffer");
        }
    }

    command_buffers
}
