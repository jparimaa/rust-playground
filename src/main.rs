use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::device::Queue;
use vulkano::instance;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::instance::QueueFamily;
use vulkano::sync::GpuFuture;

use std::sync::Arc;

fn main() {
    let validation_layer_properties = instance::layers_list()
        .expect("Could not get layers list")
        .find(|l| l.name() == "VK_LAYER_KHRONOS_validation")
        .expect("Validation layer not available");
    let layers_list = vec![validation_layer_properties.name()];

    let instance: Arc<Instance> = Instance::new(None, &InstanceExtensions::none(), layers_list)
        .expect("Failed to create an instance");

    let physical_device: PhysicalDevice = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("No physical device available");

    let queue_family: QueueFamily = physical_device
        .queue_families()
        .find(|&q| q.supports_graphics())
        .expect("Could not find a graphical queue family");

    let (device, mut queues) = {
        Device::new(
            physical_device,
            &Features::none(),
            &DeviceExtensions::none(),
            [(queue_family, 0.5)].iter().cloned(),
        )
        .expect("Failed to create a device")
    };

    let queue: Arc<Queue> = queues.next().expect("Failed to get a queue");

    let src_content = 0..64;
    let src_buffer =
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, src_content)
            .expect("Failed to create a buffer");

    let dst_content = (0..64).map(|_| 0);
    let dst_buffer =
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, dst_content)
            .expect("Failed to create a buffer");

    let mut cmd_buffer_builder = AutoCommandBufferBuilder::new(device.clone(), queue.family())
        .expect("Failed to create a command buffer builder");
    cmd_buffer_builder
        .copy_buffer(src_buffer.clone(), dst_buffer.clone())
        .unwrap();
    let command_buffer = cmd_buffer_builder
        .build()
        .expect("Failed to build a command buffer");

    let finished = command_buffer
        .execute(queue.clone())
        .expect("Failed to execute a command buffer");
    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    let src_content = src_buffer.read().unwrap();
    let dest_content = dst_buffer.read().unwrap();
    assert_eq!(&*src_content, &*dest_content);
}
