use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::pipeline_layout::PipelineLayoutAbstract;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::Dimensions;
use vulkano::image::StorageImage;
use vulkano::instance;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::instance::QueueFamily;
use vulkano::pipeline::ComputePipeline;
use vulkano::sync::GpuFuture;

use image::ImageBuffer;
use image::Rgba;

use std::sync::Arc;

mod mandelbrot {
    vulkano_shaders::shader! {
        ty: "compute",
        src: "
#version 450

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

void main() {
const vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
const vec2 c = (norm_coordinates - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);
vec2 z = vec2(0.0, 0.0);
float i;
for (i = 0.0; i < 1.0; i += 0.005) {
    z = vec2(
        z.x * z.x - z.y * z.y + c.x,
        z.y * z.x + z.x * z.y + c.y
    );

    if (length(z) > 4.0) {
        break;
    }
}

vec4 to_write = vec4(vec3(i), 1.0);
imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
}"
    }
}

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

    let image: Arc<StorageImage<Format>> = StorageImage::new(
        device.clone(),
        Dimensions::Dim2d {
            width: 1024,
            height: 1024,
        },
        Format::R8G8B8A8Unorm,
        Some(queue.family()),
    )
    .expect("Failed to create an image");

    let shader =
        mandelbrot::Shader::load(device.clone()).expect("Failed to create a shader module");

    let compute_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &shader.main_entry_point(), &(), None)
            .expect("Failed to create compute pipeline"),
    );

    let descriptor_set = Arc::new(
        PersistentDescriptorSet::start(
            compute_pipeline
                .layout()
                .descriptor_set_layout(0)
                .unwrap()
                .clone(),
        )
        .add_image(image.clone())
        .unwrap()
        .build()
        .unwrap(),
    );

    let dst_buffer: Arc<CpuAccessibleBuffer<[u8]>> = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        false,
        (0..1024 * 1024 * 4).map(|_| 0u8),
    )
    .expect("Failed to create a GPU buffer");

    let mut builder: AutoCommandBufferBuilder =
        AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
    builder
        .dispatch(
            [1024 / 8, 1024 / 8, 1],
            compute_pipeline.clone(),
            descriptor_set.clone(),
            (),
        )
        .unwrap()
        .copy_image_to_buffer(image.clone(), dst_buffer.clone())
        .unwrap();
    let command_buffer = builder.build().unwrap();

    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    let buffer_content = dst_buffer.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
    image.save("image.png").unwrap();
}
