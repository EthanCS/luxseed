pub mod define;
pub mod enums;
pub mod flag;
pub mod vulkan;

use anyhow::Result;
use luxseed_utility::pool::Handle;

use define::*;
use enums::*;
use flag::*;
use vulkan::VulkanBackend;

pub fn create_render_backend(
    backend: BackendType,
    desc: RenderBackendCreateDesc,
) -> Result<Box<dyn RenderBackend>> {
    match backend {
        BackendType::Vulkan => Ok(Box::new(VulkanBackend::new(desc)?)),
        _ => anyhow::bail!("Unsupported RHI backend type"),
    }
}

/// The RenderBackend trait defines the interface for a render backend.
pub trait RenderBackend {
    /// Gets the type of the backend.
    ///
    /// # Returns
    ///
    /// The type of the backend.
    fn get_backend_type(&self) -> BackendType;

    /// Gets supported formats from the given candidates. The candidates are sorted by priority.
    ///
    /// # Arguments
    ///
    /// * `candidates` - A slice of format candidates.
    /// * `tiling` - The tiling mode.
    /// * `feature` - The format feature flags.
    ///
    /// # Returns
    ///
    /// A `Result` containing the supported format if successful, or an error message otherwise.
    fn get_supported_format_from_candidates(
        &self,
        candidates: &[Format],
        tiling: ImageTiling,
        feature: FormatFeatureFlags,
    ) -> Result<Format>;

    /// Enumerates the adapter infos.
    ///
    /// # Returns
    ///
    /// A slice of adapter infos.
    fn enumerate_adapter_infos(&self) -> &[AdapterInfo];

    fn is_device_created(&self) -> bool;
    fn create_device(&mut self, adapter_index: usize) -> Result<()>;
    fn destroy_device(&mut self) -> Result<()>;
    fn device_wait_idle(&self) -> Result<()>;

    // Fence
    fn create_fence(&mut self, signal: bool) -> Result<Handle<Fence>>;
    fn destroy_fence(&mut self, handle: Handle<Fence>) -> Result<()>;
    fn wait_for_fences(&self, fences: &[Handle<Fence>], wait_all: bool, timeout: u64)
        -> Result<()>;
    fn reset_fences(&self, fences: &[Handle<Fence>]) -> Result<()>;

    // Semaphore
    fn create_semaphore(&mut self) -> Result<Handle<Semaphore>>;
    fn destroy_semaphore(&mut self, handle: Handle<Semaphore>) -> Result<()>;

    // Queue
    fn get_queue(&self, queue_type: QueueType) -> Result<Handle<Queue>>;
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

    /// Creates a new swapchain with the given description and returns a handle to it.
    ///
    /// # Arguments
    ///
    /// * `desc` - A reference to the description of the swapchain to create.
    ///
    /// # Returns
    ///
    /// A `Result` containing a handle to the created swapchain if successful, or an error message otherwise.
    fn create_swapchain(&mut self, desc: SwapchainCreateDesc) -> Result<Handle<Swapchain>>;

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
    fn acquire_swapchain_next_image(
        &self,
        handle: Handle<Swapchain>,
        timeout: u64,
        semaphore: Handle<Semaphore>,
        fence: Option<Handle<Fence>>,
    ) -> Result<(usize, bool)>;

    /// Gets the back buffer of the swapchain.
    ///
    /// # Arguments
    ///
    /// * `handle` - A handle to the swapchain.
    /// * `index` - The index of the back buffer.
    ///
    /// # Returns
    ///
    /// A handle to the back buffer.
    fn get_swapchain_back_buffer(
        &self,
        handle: Handle<Swapchain>,
        index: usize,
    ) -> Result<Handle<Image>>;

    fn get_swapchain_image_count(&self, handle: Handle<Swapchain>) -> Result<u8>;

    fn destroy_swapchain(&mut self, swapchain: Handle<Swapchain>) -> Result<()>;

    fn create_descriptor_set_layout(
        &mut self,
        desc: &DescriptorSetLayoutCreateDesc,
    ) -> Result<Handle<DescriptorSetLayout>>;
    fn destroy_descriptor_set_layout(&mut self, handle: Handle<DescriptorSetLayout>) -> Result<()>;
    fn create_descriptor_pool(
        &mut self,
        desc: &DescriptorPoolCreateDesc,
    ) -> Result<Handle<DescriptorPool>>;
    fn destroy_descriptor_pool(&mut self, handle: Handle<DescriptorPool>) -> Result<()>;

    fn create_descriptor_set(
        &mut self,
        desc: &DescriptorSetCreateDesc,
    ) -> Result<Handle<DescriptorSet>>;

    fn destroy_descriptor_sets(&mut self, sets: &[Handle<DescriptorSet>]) -> Result<()>;

    // Image / Image View
    fn create_image(&mut self, desc: &ImageCreateDesc) -> Result<Handle<Image>>;
    fn destroy_image(&mut self, handle: Handle<Image>) -> Result<()>;
    fn create_image_view(
        &mut self,
        image: Handle<Image>,
        desc: &ImageViewCreateDesc,
    ) -> Result<Handle<ImageView>>;
    fn destroy_image_view(&mut self, handle: Handle<ImageView>) -> Result<()>;

    fn create_sampler(&mut self, desc: &SamplerCreateDesc) -> Result<Handle<Sampler>>;
    fn destroy_sampler(&mut self, handle: Handle<Sampler>) -> Result<()>;

    // Shader
    fn create_shader_module(&mut self, desc: &ShaderModuleCreation) -> Result<Handle<Shader>>;
    fn destroy_shader_module(&mut self, shader_module: Handle<Shader>) -> Result<()>;

    // Buffer
    fn create_buffer(&mut self, desc: &BufferCreateDesc) -> Result<Handle<Buffer>>;

    fn destroy_buffer(&mut self, buffer: Handle<Buffer>) -> Result<()>;

    fn get_buffer_mapped_slice_mut(&mut self, buffer: Handle<Buffer>) -> Result<&mut [u8]>;

    fn create_pipeline_layout(
        &mut self,
        desc: &PipelineLayoutCreateDesc,
    ) -> Result<Handle<PipelineLayout>>;

    fn destroy_pipeline_layout(&mut self, pipeline_layout: Handle<PipelineLayout>) -> Result<()>;

    /// Creates a new raster pipeline with the given description and returns a handle to it.
    ///
    /// # Arguments
    ///
    /// * `desc` - A reference to the description of the pipeline to create.
    ///
    /// # Returns
    ///
    /// A `Result` containing a handle to the created pipeline if successful, or an error message otherwise.
    fn create_raster_pipeline(
        &mut self,
        desc: &RasterPipelineCreateDesc,
    ) -> Result<Handle<RasterPipeline>>;

    fn destroy_raster_pipeline(&mut self, raster_pipeline: Handle<RasterPipeline>) -> Result<()>;

    // Render pass
    fn create_render_pass(&mut self, output: &RenderPassOutput) -> Result<Handle<RenderPass>>;
    fn destroy_render_pass(&mut self, handle: Handle<RenderPass>) -> Result<()>;

    // Framebuffer
    fn create_framebuffer(
        &mut self,
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
    fn cmd_begin(&self, cb: Handle<CommandBuffer>, desc: CommandBufferBeginDesc) -> Result<()>;
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
    fn cmd_pipeline_barrier(
        &self,
        cb: Handle<CommandBuffer>,
        src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags,
        image_memory_barriers: &[ImageMemoryBarrier],
    ) -> Result<()>;
    fn cmd_bind_descriptor_sets(
        &self,
        cb: Handle<CommandBuffer>,
        bind_point: PipelineBindPoint,
        pipeline_layout: Handle<PipelineLayout>,
        first_set: u32,
        descriptor_sets: &[Handle<DescriptorSet>],
        dynamic_offsets: &[u32],
    ) -> Result<()>;

    fn cmd_bind_vertex_buffers(
        &self,
        cb: Handle<CommandBuffer>,
        first_binding: u32,
        buffers: &[Handle<Buffer>],
        offsets: &[u64],
    ) -> Result<()>;
    fn cmd_bind_index_buffer(
        &self,
        cb: Handle<CommandBuffer>,
        buffer: Handle<Buffer>,
        offset: u64,
        index_type: IndexType,
    ) -> Result<()>;
    fn cmd_copy_buffer(
        &self,
        cb: Handle<CommandBuffer>,
        src: Handle<Buffer>,
        dst: Handle<Buffer>,
        regions: &[BufferCopyRegion],
    ) -> Result<()>;
    fn cmd_copy_buffer_to_image(
        &self,
        cb: Handle<CommandBuffer>,
        src: Handle<Buffer>,
        dst: Handle<Image>,
        dst_image_layout: ImageLayout,
        regions: &[BufferImageCopyRegion],
    ) -> Result<()>;
    fn cmd_draw(
        &self,
        cb: Handle<CommandBuffer>,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<()>;
    fn cmd_draw_indexed(
        &self,
        cb: Handle<CommandBuffer>,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
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
