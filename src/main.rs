use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::instance;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;

fn main() {
    let validation_layer_properties = instance::layers_list()
        .expect("Could not get layers list")
        .find(|l| l.name() == "VK_LAYER_KHRONOS_validation")
        .expect("Validation layer not available");
    let layers_list = vec![validation_layer_properties.name()];

    let instance = Instance::new(None, &InstanceExtensions::none(), layers_list)
        .expect("Failed to create instance");

    let physical_device = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("No physical device available");

    let queue_family = physical_device
        .queue_families()
        .find(|&q| q.supports_graphics())
        .expect("Could not find a graphical queue family");

    let (_device, mut queues) = {
        Device::new(
            physical_device,
            &Features::none(),
            &DeviceExtensions::none(),
            [(queue_family, 0.5)].iter().cloned(),
        )
        .expect("Failed to create device")
    };

    let _queue = queues.next().unwrap();
}
