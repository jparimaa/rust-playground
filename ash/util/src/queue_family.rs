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
