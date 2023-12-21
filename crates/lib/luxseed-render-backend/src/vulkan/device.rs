use anyhow::{Context, Result};
use ash::{extensions::khr, vk};
use gpu_allocator::vulkan::*;
use smallvec::SmallVec;
use std::{collections::HashMap, ffi::CStr, mem::ManuallyDrop};

use crate::{
    define::*,
    enums::*,
    impl_handle,
    pool::{Handle, Pool},
};

use super::{
    command::VulkanCommandBuffer,
    framebuffer::VulkanFramebufferDesc,
    instance::VulkanInstance,
    render_pass::VulkanRenderPassOutput,
    swapchain::VulkanSwapchain,
    sync::{VulkanFence, VulkanSemaphore},
};

#[derive(Clone)]
pub struct VulkanAdapter {
    pub raw: vk::PhysicalDevice,
    pub properties: vk::PhysicalDeviceProperties,
    pub features: vk::PhysicalDeviceFeatures,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub queue_family_properties: Vec<vk::QueueFamilyProperties>,
}

impl VulkanAdapter {
    pub fn new(instance: &VulkanInstance, physical_device: vk::PhysicalDevice) -> Self {
        let properties = unsafe { instance.raw.get_physical_device_properties(physical_device) };
        let features = unsafe { instance.raw.get_physical_device_features(physical_device) };
        let memory_properties =
            unsafe { instance.raw.get_physical_device_memory_properties(physical_device) };
        let queue_family_properties =
            unsafe { instance.raw.get_physical_device_queue_family_properties(physical_device) };

        Self {
            raw: physical_device,
            properties,
            features,
            memory_properties,
            queue_family_properties,
        }
    }
}

impl AdapterInfo {
    pub fn from_vulkan(adapter: &VulkanAdapter) -> Self {
        Self {
            api_version: (adapter.properties.api_version),
            driver_version: (adapter.properties.driver_version),
            vendor_id: (adapter.properties.vendor_id),
            device_id: (adapter.properties.device_id),
            device_type: match adapter.properties.device_type {
                vk::PhysicalDeviceType::OTHER => AdapterType::Other,
                vk::PhysicalDeviceType::INTEGRATED_GPU => AdapterType::IntegratedGPU,
                vk::PhysicalDeviceType::DISCRETE_GPU => AdapterType::DiscreteGPU,
                vk::PhysicalDeviceType::VIRTUAL_GPU => AdapterType::VirtualGPU,
                vk::PhysicalDeviceType::CPU => AdapterType::CPU,
                _ => AdapterType::Other,
            },
            device_name: unsafe {
                CStr::from_ptr(adapter.properties.device_name.as_ptr()).to_str().unwrap().to_owned()
            },
        }
    }
}

pub struct VulkanDevice {
    allocator: ManuallyDrop<Allocator>,
    raw: ash::Device,
    adapter: VulkanAdapter,
    graphics_queue: Option<Handle<Queue>>,
    compute_queue: Option<Handle<Queue>>,
    transfer_queue: Option<Handle<Queue>>,
    present_queue: Option<Handle<Queue>>,
    pub render_pass_cache: HashMap<VulkanRenderPassOutput, ash::vk::RenderPass>,
    pub framebuffer_cache: HashMap<VulkanFramebufferDesc, ash::vk::Framebuffer>,
}

impl VulkanDevice {
    pub fn new(
        instance: &VulkanInstance,
        adapter: &VulkanAdapter,
        p_queue: &mut Pool<VulkanQueue>,
    ) -> anyhow::Result<VulkanDevice> {
        // Find Queue Family
        let mut main_queue_family_index = u32::MAX;
        let mut compute_queue_family_index = u32::MAX;
        let mut transfer_queue_family_index = u32::MAX;
        let mut compute_queue_index = u32::MAX;

        for (i, q) in adapter.queue_family_properties.iter().enumerate() {
            if q.queue_count == 0 {
                continue;
            }

            if q.queue_flags.contains(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE) {
                main_queue_family_index = i as u32;

                if q.queue_count > 1 {
                    compute_queue_family_index = i as u32;
                    compute_queue_index = 1;
                }

                continue;
            }

            // Compute queue
            if q.queue_flags.contains(vk::QueueFlags::COMPUTE)
                && compute_queue_family_index == u32::MAX
            {
                compute_queue_family_index = i as u32;
                compute_queue_index = 0;
            }

            // Transfer queue
            if q.queue_flags.contains(vk::QueueFlags::TRANSFER)
                && !q.queue_flags.contains(vk::QueueFlags::COMPUTE)
            {
                transfer_queue_family_index = i as u32;
                continue;
            }
        }

        let mut queue_infos: Vec<vk::DeviceQueueCreateInfo> = Vec::new();
        {
            let mut main_queue = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(main_queue_family_index)
                .queue_priorities(&[1.0, 1.0])
                .build();
            main_queue.queue_count = 1;
            if main_queue_family_index == compute_queue_family_index {
                main_queue.queue_count = 2;
            }
            queue_infos.push(main_queue);

            if main_queue_family_index != compute_queue_family_index {
                let mut compute_queue = vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(compute_queue_family_index)
                    .queue_priorities(&[1.0])
                    .build();
                compute_queue.queue_count = 1;
                queue_infos.push(compute_queue);
            }

            if transfer_queue_family_index < adapter.queue_family_properties.len() as u32 {
                let mut transfer_queue = vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(transfer_queue_family_index)
                    .queue_priorities(&[1.0])
                    .build();
                transfer_queue.queue_count = 1;
                queue_infos.push(transfer_queue);
            }
        }

        // Required device extensions
        let device_extensions = vec![khr::Swapchain::name().as_ptr()];

        // Enable all features
        let mut physical_features = vk::PhysicalDeviceFeatures2::builder().build();
        unsafe { instance.raw.get_physical_device_features2(adapter.raw, &mut physical_features) };

        // gpu-allocator need this feature
        let mut buffer_device_address = vk::PhysicalDeviceBufferDeviceAddressFeatures::builder()
            .buffer_device_address(true)
            .build();

        // Create device info
        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&device_extensions)
            .push_next(&mut physical_features)
            .push_next(&mut buffer_device_address)
            .build();

        // Create device
        let device = unsafe { instance.raw.create_device(adapter.raw, &device_create_info, None)? };

        // Create allocator
        let allocator_create_desc = AllocatorCreateDesc {
            instance: instance.raw.clone(),
            device: device.clone(),
            physical_device: adapter.raw,
            debug_settings: Default::default(),
            buffer_device_address: true,
            allocation_sizes: Default::default(),
        };
        let allocator = ManuallyDrop::new(Allocator::new(&allocator_create_desc)?);

        let mut ret = Self {
            raw: device,
            adapter: adapter.clone(),
            allocator,
            graphics_queue: None,
            compute_queue: None,
            transfer_queue: None,
            present_queue: None,
            render_pass_cache: Default::default(),
            framebuffer_cache: Default::default(),
        };

        // Get queue
        {
            let main_queue = p_queue.malloc();
            main_queue.1.init(&ret.raw, main_queue_family_index, 0);
            ret.graphics_queue = Some(main_queue.0);
            ret.present_queue = Some(main_queue.0);
        }

        {
            let compute_queue = p_queue.malloc();
            compute_queue.1.init(&ret.raw, compute_queue_family_index, compute_queue_index);
            ret.compute_queue = Some(compute_queue.0);
        }

        {
            let transfer_queue = p_queue.malloc();
            transfer_queue.1.init(&ret.raw, transfer_queue_family_index, 0);
            ret.transfer_queue = Some(transfer_queue.0);
        }

        Ok(ret)
    }

    #[inline]
    pub fn raw(&self) -> &ash::Device {
        &self.raw
    }

    #[inline]
    pub fn get_adapter(&self) -> &VulkanAdapter {
        &self.adapter
    }

    #[inline]
    pub fn get_allocator(&self) -> &Allocator {
        &self.allocator
    }

    #[inline]
    pub fn get_mut_allocator(&mut self) -> &mut Allocator {
        &mut self.allocator
    }

    #[inline]
    pub fn wait_idle(&self) -> anyhow::Result<()> {
        unsafe { self.raw().device_wait_idle()? };
        Ok(())
    }

    #[inline]
    pub fn wait_for_fences(
        &self,
        fences: &[Handle<Fence>],
        wait_all: bool,
        timeout: u64,
        p_fence: &Pool<VulkanFence>,
    ) -> anyhow::Result<()> {
        let mut raw_fences = SmallVec::<[vk::Fence; 4]>::new();
        for fence in fences {
            raw_fences.push(p_fence.get(*fence).context("Fence not found.")?.raw);
        }
        unsafe { self.raw().wait_for_fences(&raw_fences, wait_all, timeout)? };
        Ok(())
    }

    #[inline]
    pub fn reset_fences(
        &self,
        fences: &[Handle<Fence>],
        p_fence: &Pool<VulkanFence>,
    ) -> anyhow::Result<()> {
        let mut raw_fences = SmallVec::<[vk::Fence; 4]>::new();
        for fence in fences {
            raw_fences.push(p_fence.get(*fence).context("Fence not found.")?.raw);
        }
        unsafe {
            self.raw().reset_fences(&raw_fences)?;
        }
        Ok(())
    }

    #[inline]
    pub fn get_queue(&self, _type: QueueType) -> Result<Handle<Queue>> {
        match _type {
            QueueType::Graphics => self.graphics_queue.context("Graphics queue not found."),
            QueueType::Compute => self.compute_queue.context("Compute queue not found"),
            QueueType::Transfer => self.transfer_queue.context("Transfer queue not found"),
            QueueType::Present => self.present_queue.context("Present queue not found"),
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.allocator);
            self.raw.destroy_device(None);
        }
        self.graphics_queue = None;
        self.compute_queue = None;
        self.transfer_queue = None;
        self.present_queue = None;
    }
}

#[derive(Default, Clone, Copy)]
pub struct VulkanQueue {
    pub handle: Option<Handle<Queue>>,
    pub raw: vk::Queue,
    pub family_index: u32,
}
impl_handle!(VulkanQueue, Queue, handle);

impl VulkanQueue {
    #[inline]
    pub fn init(&mut self, device: &ash::Device, queue_family_index: u32, queue_index: u32) {
        self.raw = unsafe { device.get_device_queue(queue_family_index, queue_index) };
        self.family_index = queue_family_index;
    }

    #[inline]
    pub fn wait_idle(&self, device: &ash::Device) -> anyhow::Result<()> {
        unsafe { device.queue_wait_idle(self.raw)? };
        Ok(())
    }

    pub fn submit(
        &self,
        device: &VulkanDevice,
        desc: &QueueSubmitDesc,
        p_fence: &Pool<VulkanFence>,
        p_semaphore: &Pool<VulkanSemaphore>,
        p_command_buffer: &Pool<VulkanCommandBuffer>,
    ) -> anyhow::Result<()> {
        let fence = if let Some(f) = desc.fence {
            p_fence.get(f).context("Fence not found.")?.raw
        } else {
            ash::vk::Fence::null()
        };

        let mut cbs = SmallVec::<[vk::CommandBuffer; 4]>::new();
        let mut wait = SmallVec::<[vk::Semaphore; 4]>::new();
        let mut stage = SmallVec::<[vk::PipelineStageFlags; 4]>::new();
        let mut signal = SmallVec::<[vk::Semaphore; 4]>::new();

        for cb in desc.command_buffer.iter() {
            cbs.push(p_command_buffer.get(*cb).unwrap().raw);
        }

        if let Some(wait_semaphores) = desc.wait_semaphore {
            for s in wait_semaphores {
                wait.push(p_semaphore.get(*s).unwrap().raw);
            }
        }

        if let Some(wait_dst_stage_mask) = desc.wait_stage {
            for s in wait_dst_stage_mask {
                stage.push((*s).into());
            }
        }

        if let Some(signal_semaphores) = desc.finish_semaphore {
            for s in signal_semaphores {
                signal.push(p_semaphore.get(*s).unwrap().raw);
            }
        }

        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&cbs)
            .wait_semaphores(&wait)
            .wait_dst_stage_mask(&stage)
            .signal_semaphores(&signal)
            .build();

        unsafe {
            device.raw().queue_submit(self.raw, &[submit_info], fence)?;
        }
        Ok(())
    }

    pub fn present(
        &self,
        desc: &QueuePresentDesc,
        p_swapchain: &Pool<VulkanSwapchain>,
        p_semaphore: &Pool<VulkanSemaphore>,
    ) -> anyhow::Result<bool> {
        let swapchain = p_swapchain.get(desc.swapchain).context("Swapchain not found.")?;
        let wait_semaphores = desc
            .wait_semaphores
            .iter()
            .map(|s| p_semaphore.get(*s).unwrap().raw)
            .collect::<Vec<_>>();
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&[swapchain.raw])
            .image_indices(&[desc.image_index])
            .build();
        let res =
            unsafe { swapchain.loader.as_ref().unwrap().queue_present(self.raw, &present_info)? };
        Ok(res)
    }
}
