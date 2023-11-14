use ash::vk;
use std::ffi::CString;

use crate::{
    define::{Device, Shader, ShaderModuleCreation},
    impl_handle,
    pool::{Handle, Handled},
};

use super::device::VulkanDevice;

#[derive(Default)]
pub struct VulkanShader {
    pub raw: vk::ShaderModule,
    pub handle: Option<Handle<Shader>>,
    pub stage: vk::ShaderStageFlags,
    pub entry: CString,
    pub device: Option<Handle<Device>>,
}
impl_handle!(VulkanShader, Shader, handle);

impl VulkanShader {
    pub fn init(
        &mut self,
        device: &VulkanDevice,
        creation: &ShaderModuleCreation,
    ) -> anyhow::Result<()> {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(creation.code).build();
        self.raw = unsafe { device.raw().create_shader_module(&create_info, None)? };
        self.stage = creation.stage.into();
        self.entry = CString::new(creation.entry)?;
        self.device = device.get_handle();
        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_shader_module(self.raw, None);
        }
        self.raw = vk::ShaderModule::null();
        self.stage = vk::ShaderStageFlags::empty();
        self.device = None;
    }
}
