pub mod define;
pub mod enums;
pub mod pool;
pub mod vulkan;

use define::*;
use enums::*;
use pool::Handle;
use vulkan::VulkanRHI;

pub const MAX_RENDER_TARGETS: usize = 8;
pub const MAX_SHADER_STAGES: usize = 5;

pub fn rhi_create(backend: BackendType, desc: RHICreation) -> anyhow::Result<Box<dyn RHI>> {
    match backend {
        BackendType::Vulkan => Ok(Box::new(VulkanRHI::new(desc)?)),
        _ => anyhow::bail!("Unsupported RHI backend type"),
    }
}

pub trait RHI {
    // Info
    fn enum_adapters(&self) -> &[Handle<Adapter>];
    fn get_adapter_info(&self, adapter: Handle<Adapter>) -> Option<&AdapterInfo>;

    // Device
    fn create_device(&mut self, adapter: Handle<Adapter>) -> anyhow::Result<Handle<Device>>;
    fn destroy_device(&mut self, device: Handle<Device>) -> anyhow::Result<()>;
    fn wait_idle(&self, device: Handle<Device>) -> anyhow::Result<()>;

    // Fence
    fn create_fence(
        &mut self,
        device: Handle<Device>,
        signal: bool,
    ) -> anyhow::Result<Handle<Fence>>;
    fn destroy_fence(&mut self, handle: Handle<Fence>) -> anyhow::Result<()>;
    fn wait_for_fences(
        &self,
        fences: &[Handle<Fence>],
        wait_all: bool,
        timeout: u64,
    ) -> anyhow::Result<()>;
    fn reset_fences(&self, fences: &[Handle<Fence>]) -> anyhow::Result<()>;

    // Semaphore
    fn create_semaphore(&mut self, device: Handle<Device>) -> anyhow::Result<Handle<Semaphore>>;
    fn destroy_semaphore(&mut self, handle: Handle<Semaphore>) -> anyhow::Result<()>;

    // Queue
    fn get_queue(
        &mut self,
        device: Handle<Device>,
        queue_type: QueueType,
    ) -> anyhow::Result<Handle<Queue>>;
    fn queue_submit(&self, handle: Handle<Queue>, desc: &QueueSubmitDesc) -> anyhow::Result<()>;
    fn queue_present(&self, handle: Handle<Queue>, desc: &QueuePresentDesc)
        -> anyhow::Result<bool>;
    fn wait_queue_idle(&self, handle: Handle<Queue>) -> anyhow::Result<()>;

    // Surface
    fn create_surface(&mut self, desc: SurfaceCreateDesc) -> anyhow::Result<Handle<Surface>>;
    fn destroy_surface(&mut self, surface: Handle<Surface>) -> anyhow::Result<()>;

    // Swapchain
    fn create_swapchain(
        &mut self,
        device: Handle<Device>,
        creation: SwapchainCreation,
    ) -> anyhow::Result<Handle<Swapchain>>;
    fn acquire_next_image(
        &self,
        handle: Handle<Swapchain>,
        timeout: u64,
        semaphore: Handle<Semaphore>,
        fence: Option<Handle<Fence>>,
    ) -> anyhow::Result<usize>;
    fn get_swapchain_back_buffer(
        &self,
        handle: Handle<Swapchain>,
        index: usize,
    ) -> anyhow::Result<Handle<Texture>>;
    fn get_swapchain_image_count(&self, handle: Handle<Swapchain>) -> anyhow::Result<u8>;
    fn destroy_swapchain(&mut self, swapchain: Handle<Swapchain>) -> anyhow::Result<()>;

    // Texture / Texture View
    fn create_texture(
        &mut self,
        device: Handle<Device>,
        creation: &TextureCreation,
    ) -> anyhow::Result<Handle<Texture>>;
    fn destroy_texture(&mut self, handle: Handle<Texture>) -> anyhow::Result<()>;
    fn create_texture_view(
        &mut self,
        device: Handle<Device>,
        texture: Handle<Texture>,
        creation: &TextureViewCreateDesc,
    ) -> anyhow::Result<Handle<TextureView>>;
    fn destroy_texture_view(&mut self, handle: Handle<TextureView>) -> anyhow::Result<()>;

    // Shader
    fn create_shader_module(
        &mut self,
        device: Handle<Device>,
        desc: &ShaderModuleCreation,
    ) -> anyhow::Result<Handle<Shader>>;
    fn destroy_shader_module(&mut self, shader_module: Handle<Shader>) -> anyhow::Result<()>;

    // Pipeline
    fn create_raster_pipeline(
        &mut self,
        device: Handle<Device>,
        desc: &RasterPipelineCreation,
    ) -> anyhow::Result<Handle<RasterPipeline>>;
    fn destroy_raster_pipeline(
        &mut self,
        raster_pipeline: Handle<RasterPipeline>,
    ) -> anyhow::Result<()>;

    // Render pass
    fn create_render_pass(
        &mut self,
        device: Handle<Device>,
        output: &RenderPassOutput,
    ) -> anyhow::Result<Handle<RenderPass>>;
    fn destroy_render_pass(&mut self, handle: Handle<RenderPass>) -> anyhow::Result<()>;

    // Framebuffer
    fn create_framebuffer(
        &mut self,
        device: Handle<Device>,
        creation: &FramebufferCreateDesc,
    ) -> anyhow::Result<Handle<Framebuffer>>;
    fn destroy_framebuffer(&mut self, handle: Handle<Framebuffer>) -> anyhow::Result<()>;

    // Command pool / Command buffer
    fn create_command_pool(&mut self, queue: Handle<Queue>) -> anyhow::Result<Handle<CommandPool>>;
    fn reset_command_pool(&self, command_pool: Handle<CommandPool>) -> anyhow::Result<()>;
    fn destroy_command_pool(&mut self, command_pool: Handle<CommandPool>) -> anyhow::Result<()>;
    fn create_command_buffer(
        &mut self,
        command_pool: Handle<CommandPool>,
        level: CommandBufferLevel,
    ) -> anyhow::Result<Handle<CommandBuffer>>;
    fn reset_command_buffer(
        &self,
        handle: Handle<CommandBuffer>,
        release_resources: bool,
    ) -> anyhow::Result<()>;
    fn destroy_command_buffer(
        &mut self,
        command_buffer: Handle<CommandBuffer>,
    ) -> anyhow::Result<()>;

    // CMDs
    fn cmd_begin(&self, cb: Handle<CommandBuffer>) -> anyhow::Result<()>;
    fn cmd_end(&self, cb: Handle<CommandBuffer>) -> anyhow::Result<()>;
    fn cmd_begin_render_pass(
        &self,
        cb: Handle<CommandBuffer>,
        render_pass: Handle<RenderPass>,
        framebuffer: Handle<Framebuffer>,
        clear_values: Option<&[ClearColor]>,
        clear_depth_stencil: Option<ClearDepthStencil>,
    ) -> anyhow::Result<()>;
    fn cmd_end_render_pass(&self, cb: Handle<CommandBuffer>) -> anyhow::Result<()>;
    fn cmd_bind_raster_pipeline(
        &self,
        cb: Handle<CommandBuffer>,
        pipeline: Handle<RasterPipeline>,
    ) -> anyhow::Result<()>;
    fn cmd_set_viewport(
        &self,
        cb: Handle<CommandBuffer>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) -> anyhow::Result<()>;
    fn cmd_set_scissor(
        &self,
        cb: Handle<CommandBuffer>,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()>;
    fn cmd_draw(
        &self,
        cb: Handle<CommandBuffer>,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> anyhow::Result<()>;

    // Debug
    fn cmd_begin_event(
        &self,
        command_buffer: Handle<CommandBuffer>,
        name: &str,
        color: [f32; 4],
    ) -> anyhow::Result<()>;
    fn cmd_end_event(&self, command_buffer: Handle<CommandBuffer>) -> anyhow::Result<()>;
    fn cmd_set_marker(
        &self,
        command_buffer: Handle<CommandBuffer>,
        name: &str,
        color: [f32; 4],
    ) -> anyhow::Result<()>;
}
