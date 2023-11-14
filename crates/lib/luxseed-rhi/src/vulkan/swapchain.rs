use anyhow::Result;
use ash::{
    extensions::khr,
    vk::{self, SurfaceFormatKHR},
};

use crate::{
    define::{Device, Surface, Swapchain, SwapchainCreation, Texture},
    impl_handle,
    pool::{Handle, Handled, Pool},
    vulkan::{
        device::{VulkanAdapter, VulkanQueue},
        surface::VulkanSurface,
    },
};

use super::{
    device::VulkanDevice,
    image::{VulkanImage, VulkanImageDesc},
    instance::VulkanInstance,
    sync::{VulkanFence, VulkanSemaphore},
};

#[derive(Default)]
pub struct VulkanSwapchain {
    pub handle: Option<Handle<Swapchain>>,
    pub raw: vk::SwapchainKHR,
    pub device: Option<Handle<Device>>,
    pub surface: Option<Handle<Surface>>,
    pub loader: Option<khr::Swapchain>,
    pub surface_format: SurfaceFormatKHR,
    pub back_buffers: Vec<Handle<Texture>>,
    pub image_count: u8,
}
impl_handle!(VulkanSwapchain, Swapchain, handle);

impl VulkanSwapchain {
    pub fn init(
        &mut self,
        instance: &VulkanInstance,
        device: &VulkanDevice,
        adapter: &VulkanAdapter,
        surface: &VulkanSurface,
        queue: &VulkanQueue,
        desc: SwapchainCreation,
        p_texture: &mut Pool<VulkanImage>,
    ) -> Result<()> {
        let surface_supported = unsafe {
            surface.loader.as_ref().unwrap().get_physical_device_surface_support(
                adapter.raw,
                queue.family_index,
                surface.raw,
            )?
        };
        if !surface_supported {
            return Err(anyhow::anyhow!("Err no WSI support on physical device"));
        }

        let surface_formats = unsafe {
            surface
                .loader
                .as_ref()
                .unwrap()
                .get_physical_device_surface_formats(adapter.raw, surface.raw)?
        };
        let mut surface_format = SurfaceFormatKHR {
            format: vk::Format::B8G8R8A8_UNORM,
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
        };
        for available_format in surface_formats.iter() {
            if available_format.format == desc.format.into() {
                surface_format = *available_format;
                break;
            }
        }

        let surface_capabilities = unsafe {
            surface
                .loader
                .as_ref()
                .unwrap()
                .get_physical_device_surface_capabilities(adapter.raw, surface.raw)?
        };

        let pre_transform = if surface_capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_capabilities.current_transform
        };

        let mut desired_image_count = 3.max(surface_capabilities.min_image_count);
        if surface_capabilities.max_image_count != 0 {
            desired_image_count = desired_image_count.min(surface_capabilities.max_image_count);
        }

        let extent = match surface_capabilities.current_extent.width {
            std::u32::MAX => vk::Extent2D { width: (desc.width), height: (desc.height) },
            _ => surface_capabilities.current_extent,
        };

        let present_mode_preference = if desc.vsync {
            vec![vk::PresentModeKHR::FIFO_RELAXED, vk::PresentModeKHR::FIFO]
        } else {
            vec![vk::PresentModeKHR::MAILBOX, vk::PresentModeKHR::IMMEDIATE]
        };

        let present_modes = unsafe {
            surface
                .loader
                .as_ref()
                .unwrap()
                .get_physical_device_surface_present_modes(adapter.raw, surface.raw)
        }?;

        let present_mode = present_mode_preference
            .into_iter()
            .find(|mode| present_modes.contains(mode))
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .clipped(true)
            .image_array_layers(1)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .surface(surface.raw)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .min_image_count(desired_image_count)
            .pre_transform(pre_transform)
            .present_mode(present_mode)
            .build();
        let loader = khr::Swapchain::new(&instance.raw, device.raw());
        let raw = unsafe { loader.create_swapchain(&swapchain_create_info, None)? };

        let raw_images = unsafe { loader.get_swapchain_images(raw)? };
        let mut images = Vec::new();
        for raw_image in raw_images.iter() {
            let item = p_texture.malloc();
            item.1.device = device.get_handle();
            item.1.raw = *raw_image;
            item.1.desc = VulkanImageDesc {
                format: surface_format.format,
                extent: vk::Extent3D { width: extent.width, height: extent.height, depth: 1 },
                array_layers: 1,
                ..Default::default()
            };
            item.1.views.clear();
            images.push(item.0);
        }

        self.raw = raw;
        self.device = device.get_handle();
        self.loader = Some(loader);
        self.surface = Some(desc.surface);
        self.back_buffers = images;
        self.surface_format = surface_format;
        self.image_count = desired_image_count as u8;

        Ok(())
    }

    pub fn acquire_next_image(
        &self,
        timeout: u64,
        semaphore: &VulkanSemaphore,
        fence: Option<&VulkanFence>,
    ) -> Result<usize> {
        let fence = if let Some(fence) = fence { fence.raw } else { vk::Fence::null() };

        let image_index = unsafe {
            self.loader.as_ref().unwrap().acquire_next_image(
                self.raw,
                timeout,
                semaphore.raw,
                fence,
            )?
        };
        return Ok(image_index.0 as usize);
    }

    pub fn destroy(&mut self) {
        if let Some(loader) = self.loader.as_ref() {
            unsafe {
                loader.destroy_swapchain(self.raw, None);
            }
        }
        self.raw = vk::SwapchainKHR::null();
        self.device = None;
        self.surface = None;
        self.loader = None;
        self.back_buffers.clear();
        self.surface_format = SurfaceFormatKHR::default();
        self.image_count = 0;
    }
}
