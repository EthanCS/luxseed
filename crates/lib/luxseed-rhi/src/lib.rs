pub mod define;
pub mod enums;
pub mod pool;
pub mod vulkan;

use anyhow::Result;

use define::*;
use enums::*;
use pool::Handle;
use vulkan::VulkanRHI;

pub const MAX_RENDER_TARGETS: usize = 8;
pub const MAX_SHADER_STAGES: usize = 5;

pub fn rhi_create(backend: BackendType, desc: RHICreation) -> Result<Box<dyn RHI>> {
    match backend {
        BackendType::Vulkan => Ok(Box::new(VulkanRHI::new(desc)?)),
        _ => anyhow::bail!("Unsupported RHI backend type"),
    }
}

pub trait RHI {
    // Info
    fn enumerate_adapter_infos(&self) -> &[AdapterInfo];

    // Device
    fn create_device(&mut self, adapter_index: usize) -> Result<Handle<Device>>;
    fn destroy_device(&mut self, device: Handle<Device>) -> Result<()>;
    fn wait_idle(&self, device: Handle<Device>) -> Result<()>;

    // Fence
    fn create_fence(&mut self, device: Handle<Device>, signal: bool) -> Result<Handle<Fence>>;
    fn destroy_fence(&mut self, handle: Handle<Fence>) -> Result<()>;
    fn wait_for_fences(&self, fences: &[Handle<Fence>], wait_all: bool, timeout: u64)
        -> Result<()>;
    fn reset_fences(&self, fences: &[Handle<Fence>]) -> Result<()>;

    // Semaphore
    fn create_semaphore(&mut self, device: Handle<Device>) -> Result<Handle<Semaphore>>;
    fn destroy_semaphore(&mut self, handle: Handle<Semaphore>) -> Result<()>;

    // Queue
    fn get_queue(&mut self, device: Handle<Device>, queue_type: QueueType)
        -> Result<Handle<Queue>>;
    fn queue_submit(&self, handle: Handle<Queue>, desc: &QueueSubmitDesc) -> Result<()>;

    /// Presents the swapchain.
    ///
    /// # Arguments
    ///
    /// * `handle` - A handle to the queue.
    /// * `desc` - A description of the present operation.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the swapchain is suboptimal.
    fn queue_present(&self, handle: Handle<Queue>, desc: &QueuePresentDesc) -> Result<bool>;

    fn queue_wait_idle(&self, handle: Handle<Queue>) -> Result<()>;

    // Surface
    fn create_surface(&mut self, desc: SurfaceCreateDesc) -> Result<Handle<Surface>>;
    fn destroy_surface(&mut self, surface: Handle<Surface>) -> Result<()>;

    // Swapchain
    fn create_swapchain(
        &mut self,
        device: Handle<Device>,
        creation: SwapchainCreation,
    ) -> Result<Handle<Swapchain>>;

    /// Acquires the next image from the swapchain.
    ///
    /// # Arguments
    ///
    /// * `handle` - A handle to the swapchain.
    /// * `timeout` - The timeout in nanoseconds.
    /// * `semaphore` - A handle to the semaphore.
    /// * `fence` - An optional handle to the fence.
    ///
    /// # Returns
    ///
    /// A tuple containing the index of the acquired image (`usize::MAX` means swapchain is out of date) and a boolean indicating whether the swapchain is suboptimal.
    fn acquire_next_image(
        &self,
        handle: Handle<Swapchain>,
        timeout: u64,
        semaphore: Handle<Semaphore>,
        fence: Option<Handle<Fence>>,
    ) -> Result<(usize, bool)>;

    fn get_swapchain_back_buffer(
        &self,
        handle: Handle<Swapchain>,
        index: usize,
    ) -> Result<Handle<Texture>>;
    fn get_swapchain_image_count(&self, handle: Handle<Swapchain>) -> Result<u8>;
    fn destroy_swapchain(&mut self, swapchain: Handle<Swapchain>) -> Result<()>;

    // Texture / Texture View
    fn create_texture(
        &mut self,
        device: Handle<Device>,
        creation: &TextureCreation,
    ) -> Result<Handle<Texture>>;
    fn destroy_texture(&mut self, handle: Handle<Texture>) -> Result<()>;
    fn create_texture_view(
        &mut self,
        device: Handle<Device>,
        texture: Handle<Texture>,
        creation: &TextureViewCreateDesc,
    ) -> Result<Handle<TextureView>>;
    fn destroy_texture_view(&mut self, handle: Handle<TextureView>) -> Result<()>;

    // Shader
    fn create_shader_module(
        &mut self,
        device: Handle<Device>,
        desc: &ShaderModuleCreation,
    ) -> Result<Handle<Shader>>;
    fn destroy_shader_module(&mut self, shader_module: Handle<Shader>) -> Result<()>;

    // Buffer
    fn create_buffer(
        &mut self,
        device: Handle<Device>,
        desc: &BufferCreateDesc,
    ) -> Result<Handle<Buffer>>;

    fn destroy_buffer(&mut self, buffer: Handle<Buffer>) -> Result<()>;

    /// Creates a new raster pipeline with the given description and returns a handle to it.
    ///
    /// # Arguments
    ///
    /// * `device` - A handle to the device to create the pipeline on.
    /// * `desc` - A reference to the description of the pipeline to create.
    ///
    /// # Returns
    ///
    /// A `Result` containing a handle to the created pipeline if successful, or an error message otherwise.
    fn create_raster_pipeline(
        &mut self,
        device: Handle<Device>,
        desc: &RasterPipelineCreateDesc,
    ) -> Result<Handle<RasterPipeline>>;

    fn destroy_raster_pipeline(&mut self, raster_pipeline: Handle<RasterPipeline>) -> Result<()>;

    // Render pass
    fn create_render_pass(
        &mut self,
        device: Handle<Device>,
        output: &RenderPassOutput,
    ) -> Result<Handle<RenderPass>>;
    fn destroy_render_pass(&mut self, handle: Handle<RenderPass>) -> Result<()>;

    // Framebuffer
    fn create_framebuffer(
        &mut self,
        device: Handle<Device>,
        creation: &FramebufferCreateDesc,
    ) -> Result<Handle<Framebuffer>>;
    fn destroy_framebuffer(&mut self, handle: Handle<Framebuffer>) -> Result<()>;

    // Command pool / Command buffer
    fn create_command_pool(&mut self, queue: Handle<Queue>) -> Result<Handle<CommandPool>>;
    fn reset_command_pool(&self, command_pool: Handle<CommandPool>) -> Result<()>;
    fn destroy_command_pool(&mut self, command_pool: Handle<CommandPool>) -> Result<()>;
    fn create_command_buffer(
        &mut self,
        command_pool: Handle<CommandPool>,
        level: CommandBufferLevel,
    ) -> Result<Handle<CommandBuffer>>;
    fn reset_command_buffer(
        &self,
        handle: Handle<CommandBuffer>,
        release_resources: bool,
    ) -> Result<()>;
    fn destroy_command_buffer(&mut self, command_buffer: Handle<CommandBuffer>) -> Result<()>;

    // CMDs
    fn cmd_begin(&self, cb: Handle<CommandBuffer>) -> Result<()>;
    fn cmd_end(&self, cb: Handle<CommandBuffer>) -> Result<()>;
    fn cmd_begin_render_pass(
        &self,
        cb: Handle<CommandBuffer>,
        render_pass: Handle<RenderPass>,
        framebuffer: Handle<Framebuffer>,
        clear_values: Option<&[ClearColor]>,
        clear_depth_stencil: Option<ClearDepthStencil>,
    ) -> Result<()>;
    fn cmd_end_render_pass(&self, cb: Handle<CommandBuffer>) -> Result<()>;
    fn cmd_bind_raster_pipeline(
        &self,
        cb: Handle<CommandBuffer>,
        pipeline: Handle<RasterPipeline>,
    ) -> Result<()>;
    fn cmd_set_viewport(
        &self,
        cb: Handle<CommandBuffer>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) -> Result<()>;
    fn cmd_set_scissor(
        &self,
        cb: Handle<CommandBuffer>,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<()>;
    fn cmd_bind_vertex_buffers(
        &self,
        cb: Handle<CommandBuffer>,
        first_binding: u32,
        buffers: &[Handle<Buffer>],
        offsets: &[u64],
    ) -> Result<()>;
    fn cmd_copy_buffer(
        &self,
        cb: Handle<CommandBuffer>,
        src: Handle<Buffer>,
        dst: Handle<Buffer>,
        regions: &[BufferCopyRegion],
    ) -> Result<()>;
    fn cmd_draw(
        &self,
        cb: Handle<CommandBuffer>,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<()>;

    // Debug
    fn cmd_begin_event(
        &self,
        command_buffer: Handle<CommandBuffer>,
        name: &str,
        color: [f32; 4],
    ) -> Result<()>;
    fn cmd_end_event(&self, command_buffer: Handle<CommandBuffer>) -> Result<()>;
    fn cmd_set_marker(
        &self,
        command_buffer: Handle<CommandBuffer>,
        name: &str,
        color: [f32; 4],
    ) -> Result<()>;
}
