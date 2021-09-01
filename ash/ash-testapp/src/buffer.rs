use ash::version::DeviceV1_0;
use ash::vk;

pub struct Buffer {
    device: ash::Device,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

impl Buffer {
    pub fn new(device: ash::Device, buffer: vk::Buffer, memory: vk::DeviceMemory) -> Buffer {
        Buffer { device, buffer, memory }
    }

    pub fn destroy(&self) {
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}

pub fn create_vertex_buffer(
    device: &ash::Device,
    physical_device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    command_pool: vk::CommandPool,
    submit_queue: vk::Queue,
    mesh: &util::gltf_model::Mesh,
) -> Buffer {
    let buffer_size = (std::mem::size_of_val(&mesh.vertices[0]) *  mesh.vertices.len()) as vk::DeviceSize;

    let (staging_buffer, staging_buffer_memory) = util::memory::create_buffer(
        device,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        &physical_device_memory_properties,
    );

    unsafe {
        let data_ptr = device
            .map_memory(staging_buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
            .expect("Failed to map memory") as *mut util::gltf_model::Vertex;

        data_ptr.copy_from_nonoverlapping(mesh.vertices.as_ptr(), mesh.vertices.len());

        device.unmap_memory(staging_buffer_memory);
    }

    let (vertex_buffer, vertex_buffer_memory) = util::memory::create_buffer(
        device,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        &physical_device_memory_properties,
    );

    util::memory::copy_buffer(device, submit_queue, command_pool, staging_buffer, vertex_buffer, buffer_size);

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }

    Buffer::new(device.clone(), vertex_buffer, vertex_buffer_memory)
}

pub fn create_index_buffer(
    device: &ash::Device,
    physical_device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    command_pool: vk::CommandPool,
    submit_queue: vk::Queue,
    mesh: &util::gltf_model::Mesh,
) -> Buffer {
    let buffer_size = (std::mem::size_of_val(&mesh.indices[0]) * mesh.indices.len()) as vk::DeviceSize;

    let (staging_buffer, staging_buffer_memory) = util::memory::create_buffer(
        device,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        &physical_device_memory_properties,
    );

    unsafe {
        let data_ptr = device
            .map_memory(staging_buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
            .expect("Failed to map memory") as *mut u32;

        data_ptr.copy_from_nonoverlapping(mesh.indices.as_ptr(), mesh.indices.len());

        device.unmap_memory(staging_buffer_memory);
    }

    let (index_buffer, index_buffer_memory) = util::memory::create_buffer(
        device,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        &physical_device_memory_properties,
    );

    util::memory::copy_buffer(device, submit_queue, command_pool, staging_buffer, index_buffer, buffer_size);

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }

    Buffer::new(device.clone(), index_buffer, index_buffer_memory)
}

pub fn create_uniform_buffers(
    device: &ash::Device,
    physical_device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    num_buffers: usize,
) -> Vec<Buffer> {
    let buffer_size = std::mem::size_of::<crate::data::WVPMatrices>();

    let mut buffers = vec![];

    for _ in 0..num_buffers {
        let (uniform_buffer, uniform_buffer_memory) = util::memory::create_buffer(
            device,
            buffer_size as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            physical_device_memory_properties,
        );

        buffers.push(Buffer::new(device.clone(), uniform_buffer, uniform_buffer_memory));
    }

    buffers
}
