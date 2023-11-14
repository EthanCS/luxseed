use ash::{extensions::khr, vk};

use crate::{
    define::{Surface, SurfaceCreateDesc},
    impl_handle,
    pool::Handle,
};

use super::instance::VulkanInstance;

#[derive(Default)]
pub struct VulkanSurface {
    pub handle: Option<Handle<Surface>>,
    pub raw: vk::SurfaceKHR,
    pub loader: Option<khr::Surface>,
}
impl_handle!(VulkanSurface, Surface, handle);

impl VulkanSurface {
    pub fn init(&mut self, instance: &VulkanInstance, desc: SurfaceCreateDesc) -> anyhow::Result<()> {
        self.raw = unsafe {
            ash_window::create_surface(
                &instance.entry,
                &instance.raw,
                desc.raw_display_handle,
                desc.raw_window_handle,
                None,
            )?
        };
        self.loader = Some(khr::Surface::new(&instance.entry, &instance.raw));
        Ok(())
    }

    pub fn destroy(&mut self) {
        if let Some(loader) = self.loader.as_ref() {
            unsafe {
                loader.destroy_surface(self.raw, None);
            }
        }
        self.loader = None;
        self.raw = vk::SurfaceKHR::null();
    }
}
