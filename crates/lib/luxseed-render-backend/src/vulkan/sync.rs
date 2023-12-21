use anyhow::Ok;
use ash::vk::{self};

use crate::{
    define::{Fence, Semaphore},
    impl_handle,
    pool::Handle,
};

use super::device::VulkanDevice;

#[derive(Default)]
pub struct VulkanFence {
    pub handle: Option<Handle<Fence>>,
    pub raw: vk::Fence,
}
impl_handle!(VulkanFence, Fence, handle);

impl VulkanFence {
    pub fn init(&mut self, device: &VulkanDevice, signal: bool) -> anyhow::Result<()> {
        let create_info = vk::FenceCreateInfo::builder()
            .flags(if signal {
                vk::FenceCreateFlags::SIGNALED
            } else {
                vk::FenceCreateFlags::empty()
            })
            .build();
        self.raw = unsafe { device.raw().create_fence(&create_info, None)? };
        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_fence(self.raw, None);
        }
        self.raw = vk::Fence::null();
    }
}

#[derive(Default)]
pub struct VulkanSemaphore {
    pub handle: Option<Handle<Semaphore>>,
    pub raw: vk::Semaphore,
}
impl_handle!(VulkanSemaphore, Semaphore, handle);

impl VulkanSemaphore {
    pub fn init(&mut self, device: &VulkanDevice) -> anyhow::Result<()> {
        let create_info = vk::SemaphoreCreateInfo::builder().build();
        self.raw = unsafe { device.raw().create_semaphore(&create_info, None)? };
        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_semaphore(self.raw, None);
        }
        self.raw = vk::Semaphore::null();
    }
}
