use ash::version::DeviceV1_0;
use ash::vk;

pub struct Presenter {
    length: usize,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame_index: usize,
}

impl Presenter {
    pub fn new(device: &ash::Device, swapchain_length: usize) -> Presenter {
        let mut image_available_semaphores: Vec<vk::Semaphore> = Vec::new();
        let mut render_finished_semaphores: Vec<vk::Semaphore> = Vec::new();
        let mut in_flight_fences: Vec<vk::Fence> = Vec::new();

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

        for _ in 0..swapchain_length {
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

                image_available_semaphores.push(image_available_semaphore);
                render_finished_semaphores.push(render_finished_semaphore);
                in_flight_fences.push(in_flight_fence);
            }
        }

        Presenter {
            length: swapchain_length,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame_index: 0,
        }
    }

    pub fn destroy(&self, device: &ash::Device) {
        unsafe {
            for i in 0..self.length {
                device.destroy_semaphore(self.image_available_semaphores[i], None);
                device.destroy_semaphore(self.render_finished_semaphores[i], None);
                device.destroy_fence(self.in_flight_fences[i], None);
            }
        }
    }

    pub fn present(
        &mut self,
        device: &ash::Device,
        swapchain: &crate::swapchain::Swapchain,
        command_buffers: &Vec<vk::CommandBuffer>,
        graphics_queue: ash::vk::Queue,
        present_queue: ash::vk::Queue,
    ) {
        let frame_index = self.current_frame_index;
        let in_flight_fence = self.in_flight_fences[frame_index];
        let in_flight_fence_array = [in_flight_fence];

        unsafe {
            device
                .wait_for_fences(&in_flight_fence_array, true, std::u64::MAX)
                .expect("Failed to wait for fence");
            device
                .reset_fences(&in_flight_fence_array)
                .expect("Failed to reset fence");
        }

        let (image_index, _is_sub_optimal) = unsafe {
            swapchain
                .loader
                .acquire_next_image(
                    swapchain.vk_swapchain_khr,
                    std::u64::MAX,
                    self.image_available_semaphores[frame_index],
                    vk::Fence::null(),
                )
                .expect("Failed to acquire next swapchain image")
        };

        let image_available_semaphore = [self.image_available_semaphores[frame_index]];
        let queue_completed_semaphore = [self.render_finished_semaphores[frame_index]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: std::ptr::null(),
            wait_semaphore_count: image_available_semaphore.len() as u32,
            p_wait_semaphores: image_available_semaphore.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &command_buffers[image_index as usize],
            signal_semaphore_count: queue_completed_semaphore.len() as u32,
            p_signal_semaphores: queue_completed_semaphore.as_ptr(),
        }];

        unsafe {
            device
                .queue_submit(graphics_queue, &submit_infos, in_flight_fence)
                .expect("Failed to execute queue submit");
        }

        let swapchains = [swapchain.vk_swapchain_khr];

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
            swapchain
                .loader
                .queue_present(present_queue, &present_info)
                .expect("Failed to execute queue present");
        }

        self.current_frame_index = (self.current_frame_index + 1) % self.length;
    }
}
