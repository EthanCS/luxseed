use anyhow::{Context, Ok};
use ash::vk::{self};
use luxseed_utility::impl_handle;
use luxseed_utility::pool::{Handle, Pool};
use smallvec::SmallVec;

use crate::{
    define::{Framebuffer, FramebufferCreateDesc},
    MAX_RENDER_TARGETS,
};

use super::{
    device::VulkanDevice,
    image::{VulkanImage, VulkanImageView},
};

#[derive(Default)]
pub struct VulkanFramebuffer {
    pub handle: Option<Handle<Framebuffer>>,
    pub raw: vk::Framebuffer,
    pub desc: VulkanFramebufferDesc,
}
impl_handle!(VulkanFramebuffer, Framebuffer, handle);

impl VulkanFramebuffer {
    pub fn init(&mut self, raw: vk::Framebuffer, desc: VulkanFramebufferDesc) {
        self.raw = raw;
        self.desc = desc;
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_framebuffer(self.raw, None);
        }
        self.raw = vk::Framebuffer::null();
        self.desc = Default::default();
    }
}

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct VulkanFramebufferDesc {
    pub render_pass: vk::RenderPass,
    pub num_attachments: u8,
    pub views: [vk::ImageView; MAX_RENDER_TARGETS + 1],
    pub width: u32,
    pub height: u32,
    pub layers: u32,
}

impl VulkanFramebufferDesc {
    pub fn from_create_desc(
        render_pass: vk::RenderPass,
        creation: &FramebufferCreateDesc,
        p_texture: &Pool<VulkanImage>,
        p_texture_view: &Pool<VulkanImageView>,
    ) -> anyhow::Result<Self> {
        let mut views = [ash::vk::ImageView::null(); MAX_RENDER_TARGETS + 1];
        let mut num_attachments = 0;
        let mut layers = 1;
        let mut width = 0;
        let mut height = 0;

        for view in creation.color_views.iter() {
            let view = p_texture_view.get(*view).context("Color texture view not found")?;
            views[num_attachments as usize] = view.raw;
            num_attachments += 1;
            let texture = p_texture
                .get(view.texture.context("Texture view's texture is none")?)
                .context("Texture not found")?;
            width = texture.desc.extent.width;
            height = texture.desc.extent.height;
            layers = texture.desc.array_layers as u32;
        }

        if let Some(depth_view) = creation.depth_stencil_view {
            let view =
                p_texture_view.get(depth_view).context("Depth stencil texture view not found")?;
            views[num_attachments as usize] = view.raw;
            num_attachments += 1;
        }

        Ok(Self { render_pass, num_attachments, views, width, height, layers })
    }
}

impl VulkanDevice {
    pub fn get_or_create_framebuffer(
        &mut self,
        desc: &VulkanFramebufferDesc,
    ) -> anyhow::Result<vk::Framebuffer> {
        if let Some(fb) = self.framebuffer_cache.get(&desc) {
            return Ok(*fb);
        }
        let new_fb = VulkanDevice::create_vulkan_framebuffer(self.raw(), desc)?;
        self.framebuffer_cache.insert(*desc, new_fb);
        return Ok(new_fb);
    }

    fn create_vulkan_framebuffer(
        device: &ash::Device,
        desc: &VulkanFramebufferDesc,
    ) -> anyhow::Result<vk::Framebuffer> {
        let mut attachments: SmallVec<[vk::ImageView; MAX_RENDER_TARGETS + 1]> = Default::default();
        for i in 0..desc.num_attachments {
            attachments.push(desc.views[i as usize]);
        }
        let info = vk::FramebufferCreateInfo::builder()
            .render_pass(desc.render_pass)
            .height(desc.height.into())
            .width(desc.width.into())
            .layers(desc.layers.into())
            .attachments(&attachments)
            .build();
        let raw = unsafe { device.create_framebuffer(&info, None) }?;
        Ok(raw)
    }
}
