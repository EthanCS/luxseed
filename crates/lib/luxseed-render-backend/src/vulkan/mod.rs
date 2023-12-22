pub mod buffer;
pub mod command;
pub mod descriptor;
pub mod device;
pub mod framebuffer;
pub mod image;
pub mod instance;
pub mod pipeline;
pub mod render_pass;
pub mod shader;
pub mod surface;
pub mod swapchain;
pub mod sync;
pub mod util;

use anyhow::Context;
use anyhow::Result;
use smallvec::SmallVec;
use std::ffi::CString;

use crate::define::*;
use crate::define_resource_pool;
use crate::enums::*;
use crate::flag::*;
use crate::pool::*;
use crate::{RenderBackend, RenderBackendCreateDesc};

use self::buffer::*;
use self::command::*;
use self::descriptor::*;
use self::device::*;
use self::framebuffer::*;
use self::image::*;
use self::pipeline::*;
use self::render_pass::VulkanRenderPass;
use self::shader::VulkanShader;
use self::surface::VulkanSurface;
use self::swapchain::VulkanSwapchain;
use self::sync::*;

define_resource_pool!(
    VulkanResourcePool,
    (VulkanQueue, queue, 4),
    (VulkanSurface, surface, 1),
    (VulkanSwapchain, swapchain, 1),
    (VulkanImage, image, 128),
    (VulkanImageView, image_view, 128),
    (VulkanSampler, sampler, 32),
    (VulkanShader, shader_module, 32),
    (VulkanPipelineLayout, pipeline_layout, 32),
    (VulkanRasterPipeline, raster_pipeline, 32),
    (VulkanRenderPass, render_pass, 32),
    (VulkanFramebuffer, framebuffer, 8),
    (VulkanCommandPool, command_pool, 4),
    (VulkanCommandBuffer, command_buffer, 8),
    (VulkanFence, fence, 4),
    (VulkanSemaphore, semaphore, 4),
    (VulkanBuffer, buffer, 32),
    (VulkanDescriptorSetLayout, descriptor_set_layout, 32),
    (VulkanDescriptorPool, descriptor_pool, 32),
    (VulkanDescriptorSet, descriptor_set, 32)
);

pub struct VulkanBackend {
    instance: instance::VulkanInstance,
    res_pool: VulkanResourcePool,
    adapters: Vec<VulkanAdapter>,
    adapter_infos: Vec<AdapterInfo>,
    device: Option<VulkanDevice>,
}

impl VulkanBackend {
    pub fn new(desc: RenderBackendCreateDesc) -> Result<Self> {
        let instance = instance::VulkanInstance::new(desc)?;

        // Enumerate Adapters
        let mut adapters = Vec::new();
        let mut adapter_infos = Vec::new();
        let raw_devices = unsafe { instance.raw.enumerate_physical_devices()? };
        for raw_device in raw_devices {
            let a = VulkanAdapter::new(&instance, raw_device);
            adapter_infos.push(AdapterInfo::from_vulkan(&a));
            adapters.push(a);
        }

        let resource_pool = VulkanResourcePool::new();
        Ok(Self { instance, res_pool: resource_pool, adapters, adapter_infos, device: None })
    }

    #[inline]
    pub fn get_device(&self) -> Result<&VulkanDevice> {
        self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)
    }

    #[inline]
    pub fn get_mut_device(&mut self) -> Result<&mut VulkanDevice> {
        self.device.as_mut().context(ERR_MSG_DEVICE_NOT_CREATED)
    }
}

impl Drop for VulkanBackend {
    fn drop(&mut self) {
        //todo!()
    }
}

impl RenderBackend for VulkanBackend {
    #[inline]
    fn get_backend_type(&self) -> BackendType {
        BackendType::Vulkan
    }

    fn get_supported_format_from_candidates(
        &self,
        candidates: &[Format],
        tiling: ImageTiling,
        feature: FormatFeatureFlags,
    ) -> Result<Format> {
        let adapter = self.get_device()?.get_adapter().raw;
        candidates
            .iter()
            .cloned()
            .find(|f| {
                let props = unsafe {
                    self.instance.raw.get_physical_device_format_properties(adapter, (*f).into())
                };
                match tiling {
                    ImageTiling::Linear => props.linear_tiling_features.contains(feature.into()),
                    ImageTiling::Optimal => props.optimal_tiling_features.contains(feature.into()),
                }
            })
            .ok_or_else(|| anyhow::anyhow!("No supported format found."))
    }

    #[inline]
    fn enumerate_adapter_infos(&self) -> &[AdapterInfo] {
        &self.adapter_infos
    }

    #[inline]
    fn is_device_created(&self) -> bool {
        self.device.is_some()
    }

    fn create_device(&mut self, adapter_index: usize) -> Result<()> {
        if self.is_device_created() {
            return Err(anyhow::anyhow!("Device already created."));
        }
        let adapter = self.adapters.get(adapter_index).context("Adapter not found.")?;
        self.device = Some(VulkanDevice::new(&self.instance, adapter, &mut self.res_pool.queue)?);
        Ok(())
    }

    fn destroy_device(&mut self) -> Result<()> {
        self.device.as_mut().context(ERR_MSG_DEVICE_NOT_CREATED)?.destroy();
        Ok(())
    }

    #[inline]
    fn device_wait_idle(&self) -> Result<()> {
        self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?.wait_idle()
    }

    #[inline]
    fn get_queue(&self, queue_type: QueueType) -> Result<Handle<Queue>> {
        self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?.get_queue(queue_type)
    }

    fn queue_submit(&self, handle: Handle<Queue>, desc: &QueueSubmitDesc) -> Result<()> {
        let queue = self.res_pool.queue.get(handle).context(ERR_MSG_QUEUE_NOT_FOUND)?;
        queue.submit(
            self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?,
            desc,
            &self.res_pool.fence,
            &self.res_pool.semaphore,
            &self.res_pool.command_buffer,
        )
    }

    fn queue_present(&self, handle: Handle<Queue>, desc: &QueuePresentDesc) -> Result<bool> {
        let queue = self.res_pool.queue.get(handle).context(ERR_MSG_QUEUE_NOT_FOUND)?;
        queue.present(desc, &self.res_pool.swapchain, &self.res_pool.semaphore)
    }

    fn queue_wait_idle(&self, handle: Handle<Queue>) -> Result<()> {
        let queue = self.res_pool.queue.get(handle).context(ERR_MSG_QUEUE_NOT_FOUND)?;
        queue.wait_idle(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?.raw())
    }

    fn create_fence(&mut self, signal: bool) -> Result<Handle<Fence>> {
        let item = self.res_pool.fence.malloc();
        item.1.init(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?, signal)?;
        Ok(item.0)
    }

    fn destroy_fence(&mut self, handle: Handle<Fence>) -> Result<()> {
        if let Some(fence) = self.res_pool.fence.get_mut(handle) {
            fence.destroy(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?);
            self.res_pool.fence.free(handle);
        }
        Ok(())
    }

    fn wait_for_fences(
        &self,
        handles: &[Handle<Fence>],
        wait_all: bool,
        timeout: u64,
    ) -> Result<()> {
        self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?.wait_for_fences(
            handles,
            wait_all,
            timeout,
            &self.res_pool.fence,
        )
    }

    fn reset_fences(&self, handles: &[Handle<Fence>]) -> Result<()> {
        self.device
            .as_ref()
            .context(ERR_MSG_DEVICE_NOT_CREATED)?
            .reset_fences(handles, &self.res_pool.fence)
    }

    fn create_semaphore(&mut self) -> Result<Handle<Semaphore>> {
        let item = self.res_pool.semaphore.malloc();
        item.1.init(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?)?;
        Ok(item.0)
    }

    fn destroy_semaphore(&mut self, handle: Handle<Semaphore>) -> Result<()> {
        if let Some(s) = self.res_pool.semaphore.get_mut(handle) {
            s.destroy(self.device.as_mut().context(ERR_MSG_DEVICE_NOT_CREATED)?);
            self.res_pool.semaphore.free(handle);
        }
        Ok(())
    }

    fn create_surface(&mut self, desc: SurfaceCreateDesc) -> Result<Handle<Surface>> {
        let item = self.res_pool.surface.malloc();
        item.1.init(&self.instance, desc)?;
        Ok(item.0)
    }

    fn destroy_surface(&mut self, surface: Handle<Surface>) -> Result<()> {
        if let Some(s) = self.res_pool.surface.get_mut(surface) {
            s.destroy();
            self.res_pool.surface.free(surface);
        }
        Ok(())
    }

    fn create_swapchain(&mut self, desc: SwapchainCreateDesc) -> Result<Handle<Swapchain>> {
        let item = self.res_pool.swapchain.malloc();
        item.1.init(
            &self.instance,
            self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?,
            desc,
            &self.res_pool.surface,
            &self.res_pool.queue,
            &mut self.res_pool.image,
        )?;
        Ok(item.0)
    }

    fn destroy_swapchain(&mut self, handle: Handle<Swapchain>) -> Result<()> {
        if let Some(swapchain) = self.res_pool.swapchain.get_mut(handle) {
            // Free swapchain images and views
            {
                for handle in swapchain.back_buffers.iter() {
                    let texture =
                        self.res_pool.image.get_mut(*handle).context("Image not found")?;
                    for (_, handle) in texture.views.drain() {
                        let view = self
                            .res_pool
                            .image_view
                            .get_mut(handle)
                            .context("Image view not found.")?;
                        view.destroy(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?);
                        self.res_pool.image_view.free(handle);
                    }
                    self.res_pool.image.free(*handle);
                }
            }
            swapchain.destroy();
            self.res_pool.swapchain.free(handle);
        }
        Ok(())
    }

    fn acquire_swapchain_next_image(
        &self,
        handle: Handle<Swapchain>,
        timeout: u64,
        semaphore: Handle<Semaphore>,
        fence: Option<Handle<Fence>>,
    ) -> Result<(usize, bool)> {
        let swapchain = self.res_pool.swapchain.get(handle).context("Swapchain not found.")?;
        let semaphore = self.res_pool.semaphore.get(semaphore).context("Semaphore not found.")?;
        let fence = if let Some(f) = fence {
            Some(self.res_pool.fence.get(f).context("Fence not found.")?)
        } else {
            None
        };
        Ok(swapchain.acquire_next_image(timeout, semaphore, fence)?)
    }

    fn get_swapchain_back_buffer(
        &self,
        handle: Handle<Swapchain>,
        index: usize,
    ) -> Result<Handle<Image>> {
        let swapchain = self.res_pool.swapchain.get(handle).context("Swapchain not found.")?;
        Ok(swapchain.back_buffers[index])
    }

    fn get_swapchain_image_count(&self, handle: Handle<Swapchain>) -> Result<u8> {
        let swapchain = self.res_pool.swapchain.get(handle).context("Swapchain not found.")?;
        Ok(swapchain.image_count)
    }

    fn create_descriptor_set_layout(
        &mut self,
        desc: &DescriptorSetLayoutCreateDesc,
    ) -> Result<Handle<DescriptorSetLayout>> {
        let item = self.res_pool.descriptor_set_layout.malloc();
        item.1.init(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?, desc)?;
        Ok(item.0)
    }

    fn destroy_descriptor_set_layout(&mut self, handle: Handle<DescriptorSetLayout>) -> Result<()> {
        if let Some(dsl) = self.res_pool.descriptor_set_layout.get_mut(handle) {
            dsl.destroy(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?);
            self.res_pool.descriptor_set_layout.free(handle);
        }
        Ok(())
    }

    fn create_descriptor_pool(
        &mut self,
        desc: &DescriptorPoolCreateDesc,
    ) -> Result<Handle<DescriptorPool>> {
        let item = self.res_pool.descriptor_pool.malloc();
        item.1.init(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?, desc)?;
        Ok(item.0)
    }

    fn destroy_descriptor_pool(&mut self, handle: Handle<DescriptorPool>) -> Result<()> {
        if let Some(dp) = self.res_pool.descriptor_pool.get_mut(handle) {
            dp.destroy(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?);
            self.res_pool.descriptor_pool.free(handle);
        }
        Ok(())
    }

    fn create_descriptor_set(
        &mut self,
        desc: &DescriptorSetCreateDesc,
    ) -> Result<Handle<DescriptorSet>> {
        let item = self.res_pool.descriptor_set.malloc();
        item.1.init(
            self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?,
            desc,
            &self.res_pool.descriptor_pool,
            &self.res_pool.descriptor_set_layout,
            &self.res_pool.buffer,
            &self.res_pool.image_view,
            &self.res_pool.sampler,
        )?;
        Ok(item.0)
    }

    fn destroy_descriptor_sets(&mut self, sets: &[Handle<DescriptorSet>]) -> Result<()> {
        for set in sets {
            if let Some(ds) = self.res_pool.descriptor_set.get_mut(*set) {
                ds.destroy(
                    self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?.raw(),
                    &self.res_pool.descriptor_pool,
                )?;
                self.res_pool.descriptor_set.free(*set);
            }
        }
        Ok(())
    }

    fn create_image(&mut self, desc: &ImageCreateDesc) -> Result<Handle<Image>> {
        let item = self.res_pool.image.malloc();
        item.1.init(self.device.as_mut().context("Device not created.")?, desc)?;
        Ok(item.0)
    }

    fn destroy_image(&mut self, handle: Handle<Image>) -> Result<()> {
        if let Some(v) = self.res_pool.image.get_mut(handle) {
            // Destory related views
            {
                for (_, handle) in v.views.drain() {
                    let v = self.res_pool.image_view.get_mut(handle).unwrap();
                    v.destroy(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?);
                    self.res_pool.image_view.free(handle);
                }
            }
            v.destroy(self.device.as_mut().context("Device not created.")?)?;
            self.res_pool.image.free(handle);
        }
        Ok(())
    }

    fn create_image_view(
        &mut self,
        texture: Handle<Image>,
        desc: &ImageViewCreateDesc,
    ) -> Result<Handle<ImageView>> {
        let texture = self.res_pool.image.get_mut(texture).context("Texture not found.")?;
        let desc = VulkanImageViewDesc::from_create_desc(desc, texture);
        texture.get_or_create_view(
            self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?,
            &desc,
            &mut self.res_pool.image_view,
        )
    }

    fn destroy_image_view(&mut self, handle: Handle<ImageView>) -> Result<()> {
        if let Some(v) = self.res_pool.image_view.get_mut(handle) {
            // Remove from texture
            {
                let texture = self.res_pool.image.get_mut(v.texture.unwrap()).unwrap();
                texture.views.remove(&v.desc);
            }
            v.destroy(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?);
            self.res_pool.image_view.free(handle);
        }
        Ok(())
    }

    fn create_sampler(&mut self, desc: &SamplerCreateDesc) -> Result<Handle<Sampler>> {
        let item = self.res_pool.sampler.malloc();
        item.1.init(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?, desc)?;
        Ok(item.0)
    }

    fn destroy_sampler(&mut self, handle: Handle<Sampler>) -> Result<()> {
        if let Some(s) = self.res_pool.sampler.get_mut(handle) {
            s.destroy(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?);
            self.res_pool.sampler.free(handle);
        }
        Ok(())
    }

    fn create_shader_module(&mut self, creation: &ShaderModuleCreation) -> Result<Handle<Shader>> {
        let item = self.res_pool.shader_module.malloc();
        item.1.init(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?, creation)?;
        Ok(item.0)
    }

    fn destroy_shader_module(&mut self, handle: Handle<Shader>) -> Result<()> {
        if let Some(shader) = self.res_pool.shader_module.get_mut(handle) {
            shader.destroy(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?);
            self.res_pool.shader_module.free(handle);
        }
        Ok(())
    }

    fn create_buffer(&mut self, desc: &BufferCreateDesc) -> Result<Handle<Buffer>> {
        let item = self.res_pool.buffer.malloc();
        item.1.init(self.device.as_mut().context(ERR_MSG_DEVICE_NOT_CREATED)?, desc)?;
        Ok(item.0)
    }

    fn get_buffer_mapped_slice_mut(&mut self, buffer: Handle<Buffer>) -> Result<&mut [u8]> {
        let buffer = self.res_pool.buffer.get_mut(buffer).context("Buffer not found.")?;
        let allocation = buffer.allocation.as_mut().context("Buffer not allocated.")?;
        Ok(allocation.mapped_slice_mut().context("Buffer not mapped.")?)
    }

    fn destroy_buffer(&mut self, buffer: Handle<Buffer>) -> Result<()> {
        if let Some(b) = self.res_pool.buffer.get_mut(buffer) {
            b.destroy(self.device.as_mut().context(ERR_MSG_DEVICE_NOT_CREATED)?)?;
            self.res_pool.buffer.free(buffer);
        }
        Ok(())
    }

    fn create_pipeline_layout(
        &mut self,
        desc: &PipelineLayoutCreateDesc,
    ) -> Result<Handle<PipelineLayout>> {
        let item = self.res_pool.pipeline_layout.malloc();
        item.1.init(
            self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?,
            desc,
            &self.res_pool.descriptor_set_layout,
        )?;
        Ok(item.0)
    }

    fn destroy_pipeline_layout(&mut self, pipeline_layout: Handle<PipelineLayout>) -> Result<()> {
        if let Some(pl) = self.res_pool.pipeline_layout.get_mut(pipeline_layout) {
            pl.destroy(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?);
            self.res_pool.pipeline_layout.free(pipeline_layout);
        }
        Ok(())
    }

    fn create_raster_pipeline(
        &mut self,
        desc: &RasterPipelineCreateDesc,
    ) -> Result<Handle<RasterPipeline>> {
        let render_pass = self
            .device
            .as_mut()
            .context(ERR_MSG_DEVICE_NOT_CREATED)?
            .get_or_create_render_pass(&desc.render_pass_output.into())?;
        let pipeline_layout = self
            .res_pool
            .pipeline_layout
            .get(desc.pipeline_layout)
            .context("Pipeline layout not found.")?;
        let item = self.res_pool.raster_pipeline.malloc();
        item.1.init(
            self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?,
            render_pass,
            pipeline_layout,
            desc,
            &self.res_pool.shader_module,
        )?;
        Ok(item.0)
    }

    fn destroy_raster_pipeline(&mut self, handle: Handle<RasterPipeline>) -> Result<()> {
        if let Some(pipeline) = self.res_pool.raster_pipeline.get_mut(handle) {
            pipeline.destroy(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?);
            self.res_pool.raster_pipeline.free(handle);
        }
        Ok(())
    }

    fn create_render_pass(&mut self, output: &RenderPassOutput) -> Result<Handle<RenderPass>> {
        let item = self.res_pool.render_pass.malloc();
        let output = (*output).into();
        let rp = self
            .device
            .as_mut()
            .context(ERR_MSG_DEVICE_NOT_CREATED)?
            .get_or_create_render_pass(&output)?;
        item.1.init(rp, output);
        return Ok(item.0);
    }

    fn destroy_render_pass(&mut self, handle: Handle<RenderPass>) -> Result<()> {
        if let Some(rp) = self.res_pool.render_pass.get_mut(handle) {
            rp.destroy(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?);
            self.res_pool.render_pass.free(handle);
        }
        Ok(())
    }

    fn create_framebuffer(&mut self, desc: &FramebufferCreateDesc) -> Result<Handle<Framebuffer>> {
        let rp =
            self.res_pool.render_pass.get(desc.render_pass).context("Render pass not found.")?;
        let item = self.res_pool.framebuffer.malloc();
        let desc = VulkanFramebufferDesc::from_create_desc(
            rp.raw,
            &desc,
            &self.res_pool.image,
            &self.res_pool.image_view,
        )?;
        let fb = self
            .device
            .as_mut()
            .context(ERR_MSG_DEVICE_NOT_CREATED)?
            .get_or_create_framebuffer(&desc)?;
        item.1.init(fb, desc);
        Ok(item.0)
    }

    fn destroy_framebuffer(&mut self, handle: Handle<Framebuffer>) -> Result<()> {
        if let Some(fb) = self.res_pool.framebuffer.get_mut(handle) {
            fb.destroy(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?);
            self.res_pool.framebuffer.free(handle);
        }
        Ok(())
    }

    fn reset_command_buffer(
        &self,
        handle: Handle<CommandBuffer>,
        release_resources: bool,
    ) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(handle).context("Commmand buffer not found.")?;
        cb.reset(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?, release_resources)?;
        Ok(())
    }

    fn create_command_pool(&mut self, queue: Handle<Queue>) -> Result<Handle<CommandPool>> {
        let queue = self.res_pool.queue.get(queue).context(ERR_MSG_QUEUE_NOT_FOUND)?;
        let item = self.res_pool.command_pool.malloc();
        item.1.init(queue, self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?)?;
        Ok(item.0)
    }

    fn destroy_command_pool(&mut self, handle: Handle<CommandPool>) -> Result<()> {
        if let Some(cp) = self.res_pool.command_pool.get_mut(handle) {
            cp.destroy(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?);
            self.res_pool.command_pool.free(handle);
        }
        Ok(())
    }

    fn reset_command_pool(&self, handle: Handle<CommandPool>) -> Result<()> {
        let cp = self.res_pool.command_pool.get(handle).context("Command pool not found.")?;
        cp.reset(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?)
    }

    fn create_command_buffer(
        &mut self,
        command_pool: Handle<CommandPool>,
        level: CommandBufferLevel,
    ) -> Result<Handle<CommandBuffer>> {
        let cp = self.res_pool.command_pool.get(command_pool).context("Command pool not found.")?;
        let item = self.res_pool.command_buffer.malloc();
        item.1.init(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?, cp, level)?;
        Ok(item.0)
    }

    fn destroy_command_buffer(&mut self, handle: Handle<CommandBuffer>) -> Result<()> {
        if let Some(cb) = self.res_pool.command_buffer.get_mut(handle) {
            let pool = self
                .res_pool
                .command_pool
                .get(cb.pool.unwrap())
                .context("Command pool not found.")?;
            cb.destroy(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?, pool);
            self.res_pool.command_buffer.free(handle);
        }
        Ok(())
    }

    fn cmd_begin(&self, cb: Handle<CommandBuffer>, desc: CommandBufferBeginDesc) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        cb.begin(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?, desc)
    }

    fn cmd_end(&self, cb: Handle<CommandBuffer>) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        cb.end(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?)
    }

    fn cmd_begin_render_pass(
        &self,
        cb: Handle<CommandBuffer>,
        render_pass: Handle<RenderPass>,
        framebuffer: Handle<Framebuffer>,
        clear_values: Option<&[ClearColor]>,
        clear_depth_stencil: Option<ClearDepthStencil>,
    ) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        let rp = self.res_pool.render_pass.get(render_pass).context("Render pass not found.")?;
        let fb = self.res_pool.framebuffer.get(framebuffer).context("Framebuffer not found.")?;
        cb.begin_render_pass(
            self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?,
            rp,
            fb,
            clear_values,
            clear_depth_stencil,
        )
    }

    fn cmd_end_render_pass(&self, cb: Handle<CommandBuffer>) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        cb.end_render_pass(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?)
    }

    fn cmd_bind_raster_pipeline(
        &self,
        cb: Handle<CommandBuffer>,
        pipeline: Handle<RasterPipeline>,
    ) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        let pipeline =
            self.res_pool.raster_pipeline.get(pipeline).context("Raster pipeline not found.")?;
        cb.bind_raster_pipeline(self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?, pipeline)
    }

    fn cmd_bind_descriptor_sets(
        &self,
        cb: Handle<CommandBuffer>,
        bind_point: PipelineBindPoint,
        pipeline_layout: Handle<PipelineLayout>,
        first_set: u32,
        descriptor_sets: &[Handle<DescriptorSet>],
        dynamic_offsets: &[u32],
    ) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        let pipeline_layout = self
            .res_pool
            .pipeline_layout
            .get(pipeline_layout)
            .context("Pipeline layout not found.")?;
        let mut sets = SmallVec::<[ash::vk::DescriptorSet; 4]>::new();
        for set in descriptor_sets {
            sets.push(self.res_pool.descriptor_set.get(*set).unwrap().raw);
        }
        unsafe {
            self.device
                .as_ref()
                .context(ERR_MSG_DEVICE_NOT_CREATED)?
                .raw()
                .cmd_bind_descriptor_sets(
                    cb.raw,
                    bind_point.into(),
                    pipeline_layout.raw,
                    first_set,
                    &sets,
                    dynamic_offsets,
                );
        }
        Ok(())
    }

    fn cmd_set_scissor(
        &self,
        cb: Handle<CommandBuffer>,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        cb.set_scissor(
            self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?,
            x,
            y,
            width,
            height,
        )
    }

    fn cmd_set_viewport(
        &self,
        cb: Handle<CommandBuffer>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        cb.set_viewport(
            self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?,
            x,
            y,
            width,
            height,
            min_depth,
            max_depth,
        )
    }

    fn cmd_bind_vertex_buffers(
        &self,
        cb: Handle<CommandBuffer>,
        first_binding: u32,
        buffers: &[Handle<Buffer>],
        offsets: &[u64],
    ) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        let mut v = SmallVec::<[ash::vk::Buffer; 4]>::new();
        for buffer in buffers {
            v.push(self.res_pool.buffer.get(*buffer).unwrap().raw);
        }
        unsafe {
            self.device
                .as_ref()
                .context(ERR_MSG_DEVICE_NOT_CREATED)?
                .raw()
                .cmd_bind_vertex_buffers(cb.raw, first_binding, &v, &offsets);
        }
        Ok(())
    }

    fn cmd_bind_index_buffer(
        &self,
        cb: Handle<CommandBuffer>,
        buffer: Handle<Buffer>,
        offset: u64,
        index_type: IndexType,
    ) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        let buffer = self.res_pool.buffer.get(buffer).context("Buffer not found.")?;
        unsafe {
            self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?.raw().cmd_bind_index_buffer(
                cb.raw,
                buffer.raw,
                offset,
                index_type.into(),
            );
        }
        Ok(())
    }

    fn cmd_copy_buffer(
        &self,
        cb: Handle<CommandBuffer>,
        src: Handle<Buffer>,
        dst: Handle<Buffer>,
        regions: &[BufferCopyRegion],
    ) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        let src = self.res_pool.buffer.get(src).context("Source buffer not found.")?;
        let dst = self.res_pool.buffer.get(dst).context("Destination buffer not found.")?;
        cb.copy_buffer(
            self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?,
            src,
            dst,
            regions,
        );
        Ok(())
    }

    fn cmd_pipeline_barrier(
        &self,
        cb: Handle<CommandBuffer>,
        src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags,
        image_memory_barriers: &[ImageMemoryBarrier],
    ) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        cb.pipeline_barrier(
            self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?,
            src_stage_mask,
            dst_stage_mask,
            image_memory_barriers,
            &self.res_pool.image,
        )?;
        Ok(())
    }

    fn cmd_draw(
        &self,
        cb: Handle<CommandBuffer>,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        cb.draw(
            self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?,
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        );
        Ok(())
    }

    fn cmd_draw_indexed(
        &self,
        cb: Handle<CommandBuffer>,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        cb.draw_indexed(
            self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?,
            index_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
        );
        Ok(())
    }

    fn cmd_copy_buffer_to_image(
        &self,
        cb: Handle<CommandBuffer>,
        src: Handle<Buffer>,
        dst: Handle<Image>,
        dst_image_layout: ImageLayout,
        regions: &[BufferImageCopyRegion],
    ) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        let src = self.res_pool.buffer.get(src).context("Source buffer not found.")?;
        let dst = self.res_pool.image.get(dst).context("Destination texture not found.")?;
        cb.copy_buffer_to_image(
            self.device.as_ref().context(ERR_MSG_DEVICE_NOT_CREATED)?,
            src,
            dst,
            dst_image_layout,
            regions,
        );
        Ok(())
    }

    fn cmd_begin_event(
        &self,
        cb: Handle<CommandBuffer>,
        name: &str,
        color: [f32; 4],
    ) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        if let Some(debug_utils) = &self.instance.debug_utils {
            let name = CString::new(name).unwrap();
            let info =
                ash::vk::DebugUtilsLabelEXT::builder().label_name(&name).color(color).build();
            unsafe {
                debug_utils.cmd_begin_debug_utils_label(cb.raw, &info);
            }
        }
        Ok(())
    }

    fn cmd_end_event(&self, cb: Handle<CommandBuffer>) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        if let Some(debug_utils) = &self.instance.debug_utils {
            unsafe {
                debug_utils.cmd_end_debug_utils_label(cb.raw);
            }
        }
        Ok(())
    }

    fn cmd_set_marker(&self, cb: Handle<CommandBuffer>, name: &str, color: [f32; 4]) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        if let Some(debug_utils) = &self.instance.debug_utils {
            let name = CString::new(name).unwrap();
            let info =
                ash::vk::DebugUtilsLabelEXT::builder().label_name(&name).color(color).build();
            unsafe {
                debug_utils.cmd_insert_debug_utils_label(cb.raw, &info);
            }
        }
        Ok(())
    }
}
