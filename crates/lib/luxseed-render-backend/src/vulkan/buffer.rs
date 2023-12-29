use anyhow::{Context, Result};
use ash::vk;
use gpu_allocator::vulkan::*;
use luxseed_utility::impl_handle;
use luxseed_utility::pool::Handle;

use crate::define::{Buffer, BufferCreateDesc};

use super::device::VulkanDevice;

#[derive(Default)]
pub struct VulkanBuffer {
    pub raw: vk::Buffer,
    pub handle: Option<Handle<Buffer>>,
    pub requirements: vk::MemoryRequirements,
    pub allocation: Option<Allocation>,
    pub size: u64,
}
impl_handle!(VulkanBuffer, Buffer, handle);

impl VulkanBuffer {
    pub fn init(&mut self, device: &mut VulkanDevice, desc: &BufferCreateDesc) -> Result<()> {
        let info = vk::BufferCreateInfo::builder()
            .size(desc.size)
            .usage(desc.usage.into())
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();
        let raw = unsafe { device.raw().create_buffer(&info, None)? };
        let requirements = unsafe { device.raw().get_buffer_memory_requirements(raw) };

        let mut allocation = device.get_mut_allocator().allocate(&AllocationCreateDesc {
            name: desc.name,
            requirements,
            location: desc.memory.into(),
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;

        // Bind buffer to memory
        unsafe { device.raw().bind_buffer_memory(raw, allocation.memory(), allocation.offset())? };

        // Set initial data if any
        if let Some(initial_data) = desc.initial_data {
            allocation
                .mapped_slice_mut()
                .context("Allocation has no data")?
                .copy_from_slice(initial_data);
        }
        self.allocation = Some(allocation);
        self.raw = raw;
        self.requirements = requirements;
        self.size = desc.size as u64;

        Ok(())
    }

    pub fn destroy(&mut self, device: &mut VulkanDevice) -> Result<()> {
        if let Some(allocation) = self.allocation.take() {
            device.get_mut_allocator().free(allocation)?;
        }

        unsafe {
            device.raw().destroy_buffer(self.raw, None);
        }
        self.raw = vk::Buffer::null();
        self.allocation = None;
        self.requirements = vk::MemoryRequirements::default();
        self.size = 0;

        Ok(())
    }
}
