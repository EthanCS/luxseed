use anyhow::Result;
use ash::vk;

use crate::{
    define::{Buffer, BufferCreateDesc, Device},
    impl_handle,
    pool::{Handle, Handled},
};

use super::device::VulkanDevice;

#[derive(Default)]
pub struct VulkanBuffer {
    pub raw: vk::Buffer,
    pub handle: Option<Handle<Buffer>>,
    pub device: Option<Handle<Device>>,
}
impl_handle!(VulkanBuffer, Buffer, handle);

impl VulkanBuffer {
    pub fn init(&mut self, device: &VulkanDevice, desc: &BufferCreateDesc) -> Result<()> {
        let info = vk::BufferCreateInfo::builder()
            .size(desc.size as u64)
            .usage(desc.usage.into())
            .sharing_mode(desc.sharing_mode.into())
            .build();
        self.raw = unsafe { device.raw().create_buffer(&info, None)? };
        self.device = device.get_handle();
        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_buffer(self.raw, None);
        }
        self.raw = vk::Buffer::null();
        self.device = None;
    }
}
