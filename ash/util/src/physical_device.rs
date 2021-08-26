use ash::version::InstanceV1_0;
use ash::vk;

pub fn get_physical_device(
    instance: &ash::Instance,
    surface: &crate::surface::Surface,
) -> (vk::PhysicalDevice, crate::queue_family::QueueFamilyIndices) {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices")
    };

    use std::collections::HashSet;
    let required_extensions = [String::from("VK_KHR_swapchain")]
        .iter()
        .cloned()
        .collect::<HashSet<String>>();

    use crate::swapchain;
    for physical_device in physical_devices {
        let indices = crate::queue_family::get_queue_family_indices(instance, physical_device, surface);
        let extensions_supported =
            crate::common::are_device_extensions_supported(instance, physical_device, &required_extensions);
        let swapchain_supported =
            is_swapchain_supported(swapchain::get_swapchain_support_info(physical_device, surface));

        if indices.is_complete() && extensions_supported && swapchain_supported {
            return (physical_device, indices);
        }
    }

    panic!("Failed to find a suitable GPU");
}

fn is_swapchain_supported(swapchain_support_info: crate::swapchain::SwapchainSupportInfo) -> bool {
    !swapchain_support_info.formats.is_empty() && !swapchain_support_info.present_modes.is_empty()
}
