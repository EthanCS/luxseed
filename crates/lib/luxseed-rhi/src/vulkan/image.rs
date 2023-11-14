use ash::vk;

use crate::{
    define::{Device, Texture, TextureView, TextureViewCreateDesc},
    impl_handle,
    pool::{Handle, Handled},
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
}
impl_handle!(VulkanImage, Texture, handle);

impl VulkanImage {
    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_image(self.raw, None);
        }
        self.raw = vk::Image::null();
        self.device = None;
        self.desc = Default::default();
    }
}

#[derive(Default, Clone, Copy)]
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
    pub fn init_from_raw_image(
        &mut self,
        device: &VulkanDevice,
        image: vk::Image,
        format: vk::Format,
        desc: &TextureViewCreateDesc,
    ) -> anyhow::Result<()> {
        let mut view_desc: VulkanImageViewDesc = (*desc).into();
        view_desc.format = format;

        self.init_impl(device, image, view_desc)
    }

    pub fn init(
        &mut self,
        device: &VulkanDevice,
        texture: &VulkanImage,
        desc: &TextureViewCreateDesc,
    ) -> anyhow::Result<()> {
        self.texture = texture.get_handle();

        let mut view_desc: VulkanImageViewDesc = (*desc).into();
        if view_desc.format == vk::Format::UNDEFINED {
            view_desc.format = texture.desc.format;
        }

        self.init_impl(device, texture.raw, view_desc)
    }

    fn init_impl(
        &mut self,
        device: &VulkanDevice,
        image: vk::Image,
        desc: VulkanImageViewDesc,
    ) -> anyhow::Result<()> {
        let view_info = vk::ImageViewCreateInfo::builder()
            .view_type(desc.view_type)
            .image(image)
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

        self.desc = desc;
        self.device = device.get_handle();
        self.raw = unsafe { device.raw().create_image_view(&view_info, None)? };

        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_image_view(self.raw, None);
        }
        self.device = None;
        self.raw = vk::ImageView::null();
    }
}
