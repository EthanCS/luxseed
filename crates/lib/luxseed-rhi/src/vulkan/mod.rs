pub mod buffer;
pub mod command;
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
use crate::pool::*;
use crate::{pool::Pool, RHICreation, RHI};

use self::buffer::*;
use self::command::*;
use self::device::*;
use self::framebuffer::*;
use self::image::*;
use self::pipeline::VulkanRasterPipeline;
use self::render_pass::VulkanRenderPass;
use self::shader::VulkanShader;
use self::surface::VulkanSurface;
use self::swapchain::VulkanSwapchain;
use self::sync::*;

define_resource_pool!(
    VulkanResourcePool,
    (VulkanDevice, device, 1),
    (VulkanQueue, queue, 4),
    (VulkanSurface, surface, 1),
    (VulkanSwapchain, swapchain, 1),
    (VulkanImage, texture, 128),
    (VulkanImageView, texture_view, 128),
    (VulkanShader, shader_module, 32),
    (VulkanRasterPipeline, raster_pipeline, 32),
    (VulkanRenderPass, render_pass, 32),
    (VulkanFramebuffer, framebuffer, 8),
    (VulkanCommandPool, command_pool, 4),
    (VulkanCommandBuffer, command_buffer, 8),
    (VulkanFence, fence, 4),
    (VulkanSemaphore, semaphore, 4),
    (VulkanBuffer, buffer, 32)
);

pub struct VulkanRHI {
    instance: instance::VulkanInstance,
    res_pool: VulkanResourcePool,
    adapters: Vec<VulkanAdapter>,
    adapter_infos: Vec<AdapterInfo>,
}

impl VulkanRHI {
    pub fn new(desc: RHICreation) -> Result<Self> {
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
        Ok(Self { instance, res_pool: resource_pool, adapters, adapter_infos })
    }

    fn get_device_handle_from_fences(&self, handles: &[Handle<Fence>]) -> Result<Handle<Device>> {
        let mut ret = None;
        for handle in handles {
            let fence = self.res_pool.fence.get(*handle).context("Fence not found.")?;
            let device = fence.device.unwrap();
            if ret.is_none() {
                ret = Some(device);
            } else if ret.unwrap() != device {
                return Err(anyhow::anyhow!("Fences are not from the same device."));
            }
        }
        Ok(ret.unwrap())
    }
}

impl Drop for VulkanRHI {
    fn drop(&mut self) {
        //todo!()
    }
}

impl RHI for VulkanRHI {
    fn enumerate_adapter_infos(&self) -> &[AdapterInfo] {
        &self.adapter_infos
    }

    fn create_device(&mut self, adapter_index: usize) -> Result<Handle<Device>> {
        let adapter = self.adapters.get(adapter_index).context("Adapter not found.")?;
        let item = self.res_pool.device.malloc();
        item.1.init(&self.instance, adapter, &mut self.res_pool.queue)?;
        Ok(item.0)
    }

    fn destroy_device(&mut self, handle: Handle<Device>) -> Result<()> {
        if let Some(device) = self.res_pool.device.get_mut(handle) {
            device.destroy();
            self.res_pool.device.free(handle);
        }
        Ok(())
    }

    fn wait_idle(&self, device: Handle<Device>) -> Result<()> {
        let device = self.res_pool.device.get(device).context("Device not found.")?;
        unsafe {
            device.raw().device_wait_idle()?;
        }
        Ok(())
    }

    fn get_queue(
        &mut self,
        device: Handle<Device>,
        queue_type: QueueType,
    ) -> Result<Handle<Queue>> {
        let device = self.res_pool.device.get(device).context("Device not found.")?;
        device.get_queue(queue_type)
    }

    fn queue_submit(&self, handle: Handle<Queue>, desc: &QueueSubmitDesc) -> Result<()> {
        let queue = self.res_pool.queue.get(handle).context("Queue not found.")?;
        let device =
            self.res_pool.device.get(queue.device.unwrap()).context("Device not found.")?;
        queue.submit(
            device,
            desc,
            &self.res_pool.fence,
            &self.res_pool.semaphore,
            &self.res_pool.command_buffer,
        )
    }

    fn queue_present(&self, handle: Handle<Queue>, desc: &QueuePresentDesc) -> Result<bool> {
        let queue = self.res_pool.queue.get(handle).context("Queue not found.")?;
        queue.present(desc, &self.res_pool.swapchain, &self.res_pool.semaphore)
    }

    fn queue_wait_idle(&self, handle: Handle<Queue>) -> Result<()> {
        let queue = self.res_pool.queue.get(handle).context("Queue not found.")?;
        let device =
            self.res_pool.device.get(queue.device.unwrap()).context("Device not found.")?;
        unsafe {
            device.raw().queue_wait_idle(queue.raw)?;
        }
        Ok(())
    }

    fn create_fence(&mut self, device: Handle<Device>, signal: bool) -> Result<Handle<Fence>> {
        let device = self.res_pool.device.get(device).context("Device not found.")?;
        let item = self.res_pool.fence.malloc();
        item.1.init(device, signal)?;
        Ok(item.0)
    }

    fn destroy_fence(&mut self, handle: Handle<Fence>) -> Result<()> {
        if let Some(fence) = self.res_pool.fence.get_mut(handle) {
            let device =
                self.res_pool.device.get(fence.device.unwrap()).context("Device not found.")?;
            fence.destroy(device);
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
        let device = self.get_device_handle_from_fences(handles)?;
        let device = self.res_pool.device.get(device).context("Device not found.")?;
        let fences =
            handles.iter().map(|f| self.res_pool.fence.get(*f).unwrap().raw).collect::<Vec<_>>();
        unsafe { device.raw().wait_for_fences(&fences, wait_all, timeout)? };
        Ok(())
    }

    fn reset_fences(&self, handles: &[Handle<Fence>]) -> Result<()> {
        let device = self.get_device_handle_from_fences(handles)?;
        let fences =
            handles.iter().map(|f| self.res_pool.fence.get(*f).unwrap().raw).collect::<Vec<_>>();
        let device = self.res_pool.device.get(device).context("Device not found.")?;
        unsafe { device.raw().reset_fences(&fences)? };
        Ok(())
    }

    fn create_semaphore(&mut self, device: Handle<Device>) -> Result<Handle<Semaphore>> {
        let device = self.res_pool.device.get(device).context("Device not found.")?;
        let item = self.res_pool.semaphore.malloc();
        item.1.init(device)?;
        Ok(item.0)
    }

    fn destroy_semaphore(&mut self, handle: Handle<Semaphore>) -> Result<()> {
        if let Some(s) = self.res_pool.semaphore.get_mut(handle) {
            let device =
                self.res_pool.device.get(s.device.unwrap()).context("Device not found.")?;
            s.destroy(device);
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

    fn create_swapchain(
        &mut self,
        device: Handle<Device>,
        desc: SwapchainCreation,
    ) -> Result<Handle<Swapchain>> {
        let device = self.res_pool.device.get(device).context("Device not found.")?;
        let surface: &VulkanSurface = self.res_pool.surface.get(desc.surface).unwrap();
        let queue: &VulkanQueue = self.res_pool.queue.get(device.graphics_queue.unwrap()).unwrap();
        let item = self.res_pool.swapchain.malloc();
        item.1.init(&self.instance, device, surface, queue, desc, &mut self.res_pool.texture)?;
        Ok(item.0)
    }

    fn destroy_swapchain(&mut self, handle: Handle<Swapchain>) -> Result<()> {
        if let Some(swapchain) = self.res_pool.swapchain.get_mut(handle) {
            let device =
                self.res_pool.device.get(swapchain.device.unwrap()).context("Device not found.")?;
            // Free swapchain images and views
            {
                for handle in swapchain.back_buffers.iter() {
                    let texture =
                        self.res_pool.texture.get_mut(*handle).context("Image not found")?;
                    for (_, handle) in texture.views.drain() {
                        let view = self
                            .res_pool
                            .texture_view
                            .get_mut(handle)
                            .context("Image view not found.")?;
                        view.destroy(device);
                        self.res_pool.texture_view.free(handle);
                    }
                    self.res_pool.texture.free(*handle);
                }
            }
            swapchain.destroy();
            self.res_pool.swapchain.free(handle);
        }
        Ok(())
    }

    fn acquire_next_image(
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
    ) -> Result<Handle<Texture>> {
        let swapchain = self.res_pool.swapchain.get(handle).context("Swapchain not found.")?;
        Ok(swapchain.back_buffers[index])
    }

    fn get_swapchain_image_count(&self, handle: Handle<Swapchain>) -> Result<u8> {
        let swapchain = self.res_pool.swapchain.get(handle).context("Swapchain not found.")?;
        Ok(swapchain.image_count)
    }

    fn create_texture(
        &mut self,
        device: Handle<Device>,
        desc: &TextureCreation,
    ) -> Result<Handle<Texture>> {
        todo!()
    }

    fn destroy_texture(&mut self, handle: Handle<Texture>) -> Result<()> {
        if let Some(v) = self.res_pool.texture.get_mut(handle) {
            let device =
                self.res_pool.device.get(v.device.unwrap()).context("Device not found.")?;
            // Destory related views
            {
                for (_, handle) in v.views.drain() {
                    let v = self.res_pool.texture_view.get_mut(handle).unwrap();
                    v.destroy(device);
                    self.res_pool.texture_view.free(handle);
                }
            }
            v.destroy(device);
            self.res_pool.texture.free(handle);
        }
        Ok(())
    }

    fn create_texture_view(
        &mut self,
        device: Handle<Device>,
        texture: Handle<Texture>,
        desc: &TextureViewCreateDesc,
    ) -> Result<Handle<TextureView>> {
        let device = self.res_pool.device.get(device).context("Device not found.")?;
        let texture = self.res_pool.texture.get_mut(texture).context("Texture not found.")?;
        let desc = VulkanImageViewDesc::from_create_desc(desc, texture);
        texture.get_or_create_view(device, &desc, &mut self.res_pool.texture_view)
    }

    fn destroy_texture_view(&mut self, handle: Handle<TextureView>) -> Result<()> {
        if let Some(v) = self.res_pool.texture_view.get_mut(handle) {
            let device =
                self.res_pool.device.get(v.device.unwrap()).context("Device not found.")?;
            // Remove from texture
            {
                let texture = self.res_pool.texture.get_mut(v.texture.unwrap()).unwrap();
                texture.views.remove(&v.desc);
            }
            v.destroy(device);
            self.res_pool.texture_view.free(handle);
        }
        Ok(())
    }

    fn create_shader_module(
        &mut self,
        device: Handle<Device>,
        creation: &ShaderModuleCreation,
    ) -> Result<Handle<Shader>> {
        let device = self.res_pool.device.get(device).context("Device not found.")?;
        let item = self.res_pool.shader_module.malloc();
        item.1.init(device, creation)?;
        Ok(item.0)
    }

    fn destroy_shader_module(&mut self, handle: Handle<Shader>) -> Result<()> {
        if let Some(shader) = self.res_pool.shader_module.get_mut(handle) {
            let device =
                self.res_pool.device.get(shader.device.unwrap()).context("Device not found.")?;
            shader.destroy(device);
            self.res_pool.shader_module.free(handle);
        }
        Ok(())
    }

    fn create_buffer(
        &mut self,
        device: Handle<Device>,
        desc: &BufferCreateDesc,
    ) -> Result<Handle<Buffer>> {
        let device = self.res_pool.device.get_mut(device).context("Device not found.")?;
        let item = self.res_pool.buffer.malloc();
        item.1.init(device, desc)?;
        Ok(item.0)
    }

    fn destroy_buffer(&mut self, buffer: Handle<Buffer>) -> Result<()> {
        if let Some(b) = self.res_pool.buffer.get_mut(buffer) {
            let device =
                self.res_pool.device.get_mut(b.device.unwrap()).context("Device not found.")?;
            b.destroy(device)?;
            self.res_pool.buffer.free(buffer);
        }
        Ok(())
    }

    fn create_raster_pipeline(
        &mut self,
        device: Handle<Device>,
        creation: &RasterPipelineCreateDesc,
    ) -> Result<Handle<RasterPipeline>> {
        let device = self.res_pool.device.get_mut(device).context("Device not found.")?;
        let render_pass = device.get_or_create_render_pass(&creation.render_pass_output.into())?;
        let item = self.res_pool.raster_pipeline.malloc();
        item.1.init(device, render_pass, creation, &self.res_pool.shader_module)?;
        Ok(item.0)
    }

    fn destroy_raster_pipeline(&mut self, handle: Handle<RasterPipeline>) -> Result<()> {
        if let Some(pipeline) = self.res_pool.raster_pipeline.get_mut(handle) {
            let device =
                self.res_pool.device.get(pipeline.device.unwrap()).context("Device not found.")?;
            pipeline.destroy(device);
            self.res_pool.raster_pipeline.free(handle);
        }
        Ok(())
    }

    fn create_render_pass(
        &mut self,
        device: Handle<Device>,
        output: &RenderPassOutput,
    ) -> Result<Handle<RenderPass>> {
        let d = self.res_pool.device.get_mut(device).context("Device not found.")?;
        let item = self.res_pool.render_pass.malloc();
        let output = (*output).into();
        let rp = d.get_or_create_render_pass(&output)?;
        item.1.init(rp, d, output);
        return Ok(item.0);
    }

    fn destroy_render_pass(&mut self, handle: Handle<RenderPass>) -> Result<()> {
        if let Some(rp) = self.res_pool.render_pass.get_mut(handle) {
            let device =
                self.res_pool.device.get(rp.device.unwrap()).context("Device not found.")?;
            rp.destroy(device);
            self.res_pool.render_pass.free(handle);
        }
        Ok(())
    }

    fn create_framebuffer(
        &mut self,
        device: Handle<Device>,
        desc: &FramebufferCreateDesc,
    ) -> Result<Handle<Framebuffer>> {
        let d = self.res_pool.device.get_mut(device).context("Device not found.")?;
        let rp =
            self.res_pool.render_pass.get(desc.render_pass).context("Render pass not found.")?;
        let item = self.res_pool.framebuffer.malloc();
        let desc = VulkanFramebufferDesc::from_create_desc(
            rp.raw,
            &desc,
            &self.res_pool.texture,
            &self.res_pool.texture_view,
        )?;
        let fb = d.get_or_create_framebuffer(&desc)?;
        item.1.init(fb, d, desc);
        Ok(item.0)
    }

    fn destroy_framebuffer(&mut self, handle: Handle<Framebuffer>) -> Result<()> {
        if let Some(fb) = self.res_pool.framebuffer.get_mut(handle) {
            let device =
                self.res_pool.device.get(fb.device.unwrap()).context("Device not found.")?;
            fb.destroy(device);
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
        let device = self.res_pool.device.get(cb.device.unwrap()).context("Device not found.")?;
        cb.reset(device, release_resources)?;
        Ok(())
    }

    fn create_command_pool(&mut self, queue: Handle<Queue>) -> Result<Handle<CommandPool>> {
        let queue = self.res_pool.queue.get(queue).context("Queue not found.")?;
        let device =
            self.res_pool.device.get(queue.device.unwrap()).context("Device not found.")?;
        let item = self.res_pool.command_pool.malloc();
        item.1.init(queue, device)?;
        Ok(item.0)
    }

    fn destroy_command_pool(&mut self, handle: Handle<CommandPool>) -> Result<()> {
        if let Some(cp) = self.res_pool.command_pool.get_mut(handle) {
            let device =
                self.res_pool.device.get(cp.device.unwrap()).context("Device not found.")?;
            cp.destroy(device);
            self.res_pool.command_pool.free(handle);
        }
        Ok(())
    }

    fn reset_command_pool(&self, handle: Handle<CommandPool>) -> Result<()> {
        let cp = self.res_pool.command_pool.get(handle).context("Command pool not found.")?;
        let device = self.res_pool.device.get(cp.device.unwrap()).context("Device not found.")?;
        cp.reset(device)
    }

    fn create_command_buffer(
        &mut self,
        command_pool: Handle<CommandPool>,
        level: CommandBufferLevel,
    ) -> Result<Handle<CommandBuffer>> {
        let cp = self.res_pool.command_pool.get(command_pool).context("Command pool not found.")?;
        let device = self.res_pool.device.get(cp.device.unwrap()).context("Device not found.")?;
        let item = self.res_pool.command_buffer.malloc();
        item.1.init(device, cp, level)?;
        Ok(item.0)
    }

    fn destroy_command_buffer(&mut self, handle: Handle<CommandBuffer>) -> Result<()> {
        if let Some(cb) = self.res_pool.command_buffer.get_mut(handle) {
            let device =
                self.res_pool.device.get(cb.device.unwrap()).context("Device not found.")?;
            let pool = self
                .res_pool
                .command_pool
                .get(cb.pool.unwrap())
                .context("Command pool not found.")?;
            cb.destroy(device, pool);
            self.res_pool.command_buffer.free(handle);
        }
        Ok(())
    }

    fn cmd_begin(&self, cb: Handle<CommandBuffer>) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        let device = self.res_pool.device.get(cb.device.unwrap()).context("Device not found.")?;
        cb.begin(device)
    }

    fn cmd_end(&self, cb: Handle<CommandBuffer>) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        let device = self.res_pool.device.get(cb.device.unwrap()).context("Device not found.")?;
        cb.end(device)
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
        let device = self.res_pool.device.get(cb.device.unwrap()).context("Device not found.")?;
        let rp = self.res_pool.render_pass.get(render_pass).context("Render pass not found.")?;
        let fb = self.res_pool.framebuffer.get(framebuffer).context("Framebuffer not found.")?;
        cb.begin_render_pass(device, rp, fb, clear_values, clear_depth_stencil)
    }

    fn cmd_end_render_pass(&self, cb: Handle<CommandBuffer>) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        let device = self.res_pool.device.get(cb.device.unwrap()).context("Device not found.")?;
        cb.end_render_pass(device)
    }

    fn cmd_bind_raster_pipeline(
        &self,
        cb: Handle<CommandBuffer>,
        pipeline: Handle<RasterPipeline>,
    ) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        let device = self.res_pool.device.get(cb.device.unwrap()).context("Device not found.")?;
        let pipeline =
            self.res_pool.raster_pipeline.get(pipeline).context("Raster pipeline not found.")?;
        cb.bind_raster_pipeline(device, pipeline)
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
        let device = self.res_pool.device.get(cb.device.unwrap()).context("Device not found.")?;
        cb.set_scissor(device, x, y, width, height)
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
        let device = self.res_pool.device.get(cb.device.unwrap()).context("Device not found.")?;
        cb.set_viewport(device, x, y, width, height, min_depth, max_depth)
    }

    fn cmd_bind_vertex_buffers(
        &self,
        cb: Handle<CommandBuffer>,
        first_binding: u32,
        buffers: &[Handle<Buffer>],
        offsets: &[u64],
    ) -> Result<()> {
        let cb = self.res_pool.command_buffer.get(cb).context("Command buffer not found.")?;
        let device = self.res_pool.device.get(cb.device.unwrap()).context("Device not found.")?;
        let mut v = SmallVec::<[ash::vk::Buffer; 4]>::new();
        for buffer in buffers {
            v.push(self.res_pool.buffer.get(*buffer).unwrap().raw);
        }
        unsafe {
            device.raw().cmd_bind_vertex_buffers(cb.raw, first_binding, &v, &offsets);
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
        let device = self.res_pool.device.get(cb.device.unwrap()).context("Device not found.")?;
        unsafe {
            device.raw().cmd_bind_index_buffer(cb.raw, buffer.raw, offset, index_type.into());
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
        let device = self.res_pool.device.get(cb.device.unwrap()).context("Device not found.")?;
        cb.copy_buffer(device, src, dst, regions);
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
        let device = self.res_pool.device.get(cb.device.unwrap()).context("Device not found.")?;
        cb.draw(device, vertex_count, instance_count, first_vertex, first_instance);
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
        let device = self.res_pool.device.get(cb.device.unwrap()).context("Device not found.")?;
        cb.draw_indexed(
            device,
            index_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
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
