use ash::vk;

pub struct Swapchain {
    pub loader: ash::extensions::khr::Swapchain,
    pub vk_swapchain_khr: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
}

pub struct SwapchainSupportInfo {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub fn get_swapchain_support_info(
    physical_device: vk::PhysicalDevice,
    surface: &crate::surface::Surface,
) -> SwapchainSupportInfo {
    unsafe {
        let capabilities = surface
            .loader
            .get_physical_device_surface_capabilities(physical_device, surface.vk_surface_khr)
            .expect("Failed to query for surface capabilities");
        let formats = surface
            .loader
            .get_physical_device_surface_formats(physical_device, surface.vk_surface_khr)
            .expect("Failed to query for surface formats");
        let present_modes = surface
            .loader
            .get_physical_device_surface_present_modes(physical_device, surface.vk_surface_khr)
            .expect("Failed to query for surface present mode");

        SwapchainSupportInfo { capabilities, formats, present_modes }
    }
}

pub fn create_swapchain(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    surface: &crate::surface::Surface,
) -> Swapchain {
    let swapchain_support_info = get_swapchain_support_info(physical_device, surface);
    let surface_format = choose_swapchain_format(&swapchain_support_info.formats);
    let present_mode = choose_swapchain_present_mode(&swapchain_support_info.present_modes);
    let extent = choose_swapchain_extent(&swapchain_support_info.capabilities);

    let image_count = std::cmp::min(
        swapchain_support_info.capabilities.max_image_count,
        swapchain_support_info.capabilities.min_image_count + 1,
    );

    let swapchain_create_info = vk::SwapchainCreateInfoKHR {
        s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
        p_next: std::ptr::null(),
        flags: vk::SwapchainCreateFlagsKHR::empty(),
        surface: surface.vk_surface_khr,
        min_image_count: image_count,
        image_color_space: surface_format.color_space,
        image_format: surface_format.format,
        image_extent: extent,
        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
        image_sharing_mode: vk::SharingMode::EXCLUSIVE,
        p_queue_family_indices: std::ptr::null(),
        queue_family_index_count: 0,
        pre_transform: swapchain_support_info.capabilities.current_transform,
        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
        present_mode,
        clipped: vk::TRUE,
        old_swapchain: vk::SwapchainKHR::null(),
        image_array_layers: 1,
    };

    let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
    let vk_swapchain_khr = unsafe {
        swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Failed to create Swapchain!")
    };

    let images = unsafe {
        swapchain_loader
            .get_swapchain_images(vk_swapchain_khr)
            .expect("Failed to get swapchain images")
    };

    Swapchain {
        loader: swapchain_loader,
        vk_swapchain_khr,
        format: surface_format.format,
        extent,
        images,
    }
}

fn choose_swapchain_format(available_formats: &Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
    for available_format in available_formats {
        if available_format.format == vk::Format::B8G8R8A8_SRGB
            && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        {
            return available_format.clone();
        }
    }

    return available_formats.first().unwrap().clone();
}

fn choose_swapchain_present_mode(available_present_modes: &Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
    for &available_present_mode in available_present_modes.iter() {
        if available_present_mode == vk::PresentModeKHR::MAILBOX {
            return available_present_mode;
        }
    }

    vk::PresentModeKHR::FIFO
}

fn choose_swapchain_extent(capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::max_value() {
        capabilities.current_extent
    } else {
        vk::Extent2D {
            width: num::clamp(
                crate::constants::WINDOW_WIDTH,
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: num::clamp(
                crate::constants::WINDOW_HEIGHT,
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }
}
