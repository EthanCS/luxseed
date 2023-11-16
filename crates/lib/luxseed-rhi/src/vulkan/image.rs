use ash::vk;
use std::collections::HashMap;

use crate::{
    define::{Device, Texture, TextureView, TextureViewCreateDesc},
    impl_handle,
    pool::{Handle, Handled, Pool},
};

use super::device::VulkanDevice;

#[derive(Default, Clone, Copy)]
pub struct VulkanImageDesc {
    pub flags: vk::ImageCreateFlags,
    pub image_type: vk::ImageType,
    pub format: vk::Format,
    pub extent: vk::Extent3D,
    pub samples: vk::SampleCountFlags,
    pub mip_levels: u8,
    pub array_layers: u8,
    pub tiling: vk::ImageTiling,
    pub usage: vk::ImageUsageFlags,
    pub sharing_mode: vk::SharingMode,
    pub initial_layout: vk::ImageLayout,
}

#[derive(Default)]
pub struct VulkanImage {
    pub handle: Option<Handle<Texture>>,
    pub raw: vk::Image,
    pub device: Option<Handle<Device>>,
    pub desc: VulkanImageDesc,
    pub views: HashMap<VulkanImageViewDesc, Handle<TextureView>>,
}
impl_handle!(VulkanImage, Texture, handle);

impl VulkanImage {
    pub fn get_or_create_view(
        &mut self,
        device: &VulkanDevice,
        desc: &VulkanImageViewDesc,
        p_image_view: &mut Pool<VulkanImageView>,
    ) -> anyhow::Result<Handle<TextureView>> {
        if let Some(handle) = self.views.get(desc) {
            return Ok(*handle);
        }
        let item = p_image_view.malloc();
        item.1.init(device, self, desc)?;
        self.views.insert(*desc, item.0);
        Ok(item.0)
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_image(self.raw, None);
        }
        self.views.clear();
        self.raw = vk::Image::null();
        self.device = None;
        self.desc = Default::default();
    }
}

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct VulkanImageViewDesc {
    pub view_type: vk::ImageViewType,
    pub aspect_mask: vk::ImageAspectFlags,
    pub format: vk::Format,
    pub component_r: vk::ComponentSwizzle,
    pub component_g: vk::ComponentSwizzle,
    pub component_b: vk::ComponentSwizzle,
    pub component_a: vk::ComponentSwizzle,
    pub base_mip_level: u8,
    pub level_count: u8,
    pub base_array_layer: u8,
    pub layer_count: u8,
}

impl VulkanImageViewDesc {
    pub fn from_create_desc(desc: &TextureViewCreateDesc, image: &VulkanImage) -> Self {
        let format = if let Some(f) = desc.format { f.into() } else { image.desc.format };
        Self {
            view_type: desc.view_type.into(),
            aspect_mask: desc.aspect_mask.into(),
            format: format,
            component_r: desc.component_r.into(),
            component_g: desc.component_g.into(),
            component_b: desc.component_b.into(),
            component_a: desc.component_a.into(),
            base_mip_level: desc.base_mip_level,
            level_count: desc.level_count,
            base_array_layer: desc.base_array_layer,
            layer_count: desc.layer_count,
        }
    }
}

#[derive(Default)]
pub struct VulkanImageView {
    pub handle: Option<Handle<TextureView>>,
    pub raw: vk::ImageView,
    pub device: Option<Handle<Device>>,
    pub texture: Option<Handle<Texture>>,
    pub desc: VulkanImageViewDesc,
}
impl_handle!(VulkanImageView, TextureView, handle);

impl VulkanImageView {
    pub fn init(
        &mut self,
        device: &VulkanDevice,
        texture: &VulkanImage,
        desc: &VulkanImageViewDesc,
    ) -> anyhow::Result<()> {
        let view_info = vk::ImageViewCreateInfo::builder()
            .view_type(desc.view_type)
            .image(texture.raw)
            .components(
                vk::ComponentMapping::builder()
                    .r(desc.component_r)
                    .g(desc.component_g)
                    .b(desc.component_b)
                    .a(desc.component_a)
                    .build(),
            )
            .format(desc.format)
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(desc.aspect_mask)
                    .base_mip_level(desc.base_mip_level.into())
                    .level_count(desc.level_count.into())
                    .base_array_layer(desc.base_array_layer.into())
                    .layer_count(desc.layer_count.into())
                    .build(),
            );
        self.texture = texture.get_handle();
        self.desc = *desc;
        self.device = device.get_handle();
        self.raw = unsafe { device.raw().create_image_view(&view_info, None)? };
        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_image_view(self.raw, None);
        }
        self.device = None;
        self.texture = None;
        self.desc = Default::default();
        self.raw = vk::ImageView::null();
    }
}
