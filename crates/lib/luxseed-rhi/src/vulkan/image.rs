use ash::vk;
use std::collections::HashMap;

use crate::{
    define::*,
    impl_handle,
    pool::{Handle, Handled, Pool},
};

use super::device::VulkanDevice;

#[derive(Default, Clone, Copy)]
pub struct VulkanImageDesc {
    pub image_type: vk::ImageType,
    pub format: vk::Format,
    pub extent: vk::Extent3D,
    pub samples: vk::SampleCountFlags,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub tiling: vk::ImageTiling,
    pub usage: vk::ImageUsageFlags,
}

#[derive(Default)]
pub struct VulkanImage {
    pub handle: Option<Handle<Image>>,
    pub raw: vk::Image,
    pub device: Option<Handle<Device>>,
    pub desc: VulkanImageDesc,
    pub views: HashMap<VulkanImageViewDesc, Handle<ImageView>>,
}
impl_handle!(VulkanImage, Image, handle);

impl VulkanImage {
    pub fn init(&mut self, device: &VulkanDevice, desc: &ImageCreateDesc) -> anyhow::Result<()> {
        let image_desc = VulkanImageDesc {
            image_type: desc.texture_type.into(),
            format: desc.format.into(),
            extent: vk::Extent3D {
                width: desc.extent[0],
                height: desc.extent[1],
                depth: desc.extent[2],
            },
            samples: desc.samples.into(),
            mip_levels: desc.mip_levels,
            array_layers: desc.array_layers,
            tiling: desc.tiling.into(),
            usage: desc.usage.into(),
        };
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(image_desc.image_type)
            .extent(image_desc.extent)
            .mip_levels(image_desc.mip_levels)
            .array_layers(image_desc.array_layers)
            .format(image_desc.format)
            .tiling(image_desc.tiling)
            .samples(image_desc.samples)
            .usage(image_desc.usage)
            .initial_layout(desc.initial_layout.into())
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .flags(vk::ImageCreateFlags::empty());
        self.raw = unsafe { device.raw().create_image(&image_info, None)? };
        self.device = device.get_handle();
        self.desc = image_desc;
        self.views.clear();
        Ok(())
    }

    pub fn get_or_create_view(
        &mut self,
        device: &VulkanDevice,
        desc: &VulkanImageViewDesc,
        p_image_view: &mut Pool<VulkanImageView>,
    ) -> anyhow::Result<Handle<ImageView>> {
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
    pub handle: Option<Handle<ImageView>>,
    pub raw: vk::ImageView,
    pub device: Option<Handle<Device>>,
    pub texture: Option<Handle<Image>>,
    pub desc: VulkanImageViewDesc,
}
impl_handle!(VulkanImageView, ImageView, handle);

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

#[derive(Default)]
pub struct VulkanSampler {
    pub handle: Option<Handle<Sampler>>,
    pub raw: vk::Sampler,
    pub device: Option<Handle<Device>>,
}
impl_handle!(VulkanSampler, Sampler, handle);

impl VulkanSampler {
    pub fn init(&mut self, device: &VulkanDevice, desc: &SamplerCreateDesc) -> anyhow::Result<()> {
        let mut compare_op = vk::CompareOp::ALWAYS;
        if let Some(op) = desc.compare_op {
            compare_op = op.into();
        }

        let device_max_anisotropy =
            device.adapter.as_ref().unwrap().properties.limits.max_sampler_anisotropy;
        let mut max_anisotropy = desc.max_anisotropy.unwrap_or(1.0);
        max_anisotropy = max_anisotropy.min(device_max_anisotropy);

        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(desc.mag_filter.into())
            .min_filter(desc.min_filter.into())
            .mipmap_mode(desc.mipmap_mode.into())
            .min_lod(0.0)
            .max_lod(0.0)
            .mip_lod_bias(desc.mip_lod_bias)
            .address_mode_u(desc.address_mode_u.into())
            .address_mode_v(desc.address_mode_v.into())
            .address_mode_w(desc.address_mode_w.into())
            .anisotropy_enable(desc.max_anisotropy.is_some())
            .max_anisotropy(max_anisotropy)
            .compare_enable(desc.compare_op.is_some())
            .compare_op(compare_op)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false);
        self.raw = unsafe { device.raw().create_sampler(&sampler_info, None)? };
        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_sampler(self.raw, None);
        }
        self.raw = vk::Sampler::null();
    }
}
