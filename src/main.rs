use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;

fn main() {
    let instance =
        Instance::new(None, &InstanceExtensions::none(), None).expect("failed to create instance");
    let physical_device = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("no device available");
}
