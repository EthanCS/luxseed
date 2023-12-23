extern crate shaderc;

use anyhow::{self, Ok, Result};
use luxseed_render_backend::{
    create_render_backend, define::*, enums::*, flag::*, pool::Handle, RenderBackend,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::window::Window;

pub struct RenderSystem {
    pub backend: Box<dyn RenderBackend>,
    pub surface: Handle<Surface>,
    pub graphics_queue: Handle<Queue>,

    pub image_index: usize,
    pub swapchain: Handle<Swapchain>,
    pub swapchain_render_pass: Handle<RenderPass>,
    pub swapchain_output: RenderPassOutput,
    pub swapchain_framebuffers: Vec<Handle<Framebuffer>>,

    pub frame: usize,
    pub max_frames_in_flight: usize,
    pub in_flight_fences: Vec<Handle<Fence>>,
    pub image_availables: Vec<Handle<Semaphore>>,
    pub render_finisheds: Vec<Handle<Semaphore>>,

    pub command_pool: Handle<CommandPool>,

    pub depth_image: Handle<Image>,
    pub depth_image_view: Handle<ImageView>,
}

impl RenderSystem {
    pub fn create(window: &Window) -> Result<Self> {
        let mut rhi = create_render_backend(
            BackendType::Vulkan,
            RenderBackendCreateDesc {
                app_name: "Luxseed Vulkan - Hello World",
                app_version: 0,
                enable_debugging: true,
                raw_display_handle: window.raw_display_handle(),
            },
        )?;

        rhi.create_device(0)?;

        let surface = rhi.create_surface(SurfaceCreateDesc {
            raw_display_handle: window.raw_display_handle(),
            raw_window_handle: window.raw_window_handle(),
        })?;

        let format = Format::B8G8R8A8_SRGB;
        let swapchain = rhi.create_swapchain(SwapchainCreateDesc {
            width: window.inner_size().width,
            height: window.inner_size().height,
            surface: surface,
            vsync: true,
            format,
        })?;
        let max_frames_in_flight = rhi.get_swapchain_image_count(swapchain)? as usize;
        let graphics_queue = rhi.get_queue(QueueType::Graphics)?;
        let command_pool = rhi.create_command_pool(graphics_queue).unwrap();

        let swapchain_output = RenderPassOutput::builder()
            .add_color(
                format,
                ImageLayout::PresentSrcKhr,
                RenderTargetLoadAction::Clear,
                SampleCount::Sample1,
            )
            .set_depth_stencil(
                Format::D32_SFLOAT,
                ImageLayout::DepthStencilAttachmentOptimal,
                RenderTargetLoadAction::Clear,
                RenderTargetLoadAction::DontCare,
                SampleCount::Sample1,
            )
            .build();
        let swapchain_render_pass = rhi.create_render_pass(&swapchain_output)?;

        let mut in_flight_fences = Vec::new();
        let mut image_availables = Vec::new();
        let mut render_finisheds = Vec::new();
        let mut swapchain_framebuffers = Vec::new();

        let (depth_image, depth_image_view) =
            create_depth(&mut rhi, window.inner_size().width, window.inner_size().height)?;

        transition_image_layout(
            &mut rhi,
            command_pool,
            graphics_queue,
            depth_image,
            ImageLayout::Undefined,
            ImageLayout::DepthStencilAttachmentOptimal,
            ImageAspectFlags::DEPTH,
        )?;

        for i in 0..max_frames_in_flight {
            in_flight_fences.push(rhi.create_fence(true)?);
            image_availables.push(rhi.create_semaphore()?);
            render_finisheds.push(rhi.create_semaphore()?);

            let back_buffer = rhi.get_swapchain_back_buffer(swapchain, i as usize)?;
            let view = rhi.create_image_view(
                back_buffer,
                &ImageViewCreateDesc { ..ImageViewCreateDesc::default() },
            )?;
            let fb = rhi.create_framebuffer(&FramebufferCreateDesc {
                render_pass: swapchain_render_pass,
                color_views: &[view],
                depth_stencil_view: Some(depth_image_view),
            })?;
            swapchain_framebuffers.push(fb);
        }

        Ok(Self {
            backend: rhi,
            surface,
            graphics_queue,

            image_index: 0,
            swapchain,
            swapchain_render_pass,
            swapchain_output,
            swapchain_framebuffers,

            frame: 0,
            max_frames_in_flight,
            in_flight_fences,
            image_availables,
            render_finisheds,

            command_pool,

            depth_image,
            depth_image_view,
        })
    }

    pub fn begin_frame(&mut self, width: u32, height: u32) -> Result<bool> {
        self.backend.wait_for_fences(&[self.get_in_flight_fence()], true, u64::MAX)?;

        let image_index = self.backend.acquire_swapchain_next_image(
            self.swapchain,
            u64::MAX,
            self.get_image_available_semaphore(),
            None,
        )?;

        if image_index.0 == usize::MAX {
            self.recreate_swapchain(width, height)?;
            return Ok(false);
        }

        self.image_index = image_index.0 as usize;
        self.backend.reset_fences(&[self.get_in_flight_fence()])?;

        Ok(true)
    }

    pub fn end_frame(&mut self, recreate_swapchain: bool, width: u32, height: u32) -> Result<()> {
        let suboptimal = self.backend.queue_present(
            self.graphics_queue,
            &QueuePresentDesc {
                swapchain: self.swapchain,
                image_index: self.image_index as u32,
                wait_semaphores: &[self.get_render_finished_semaphore()],
            },
        )?;

        if suboptimal || recreate_swapchain {
            self.recreate_swapchain(width, height)?;
        }

        self.frame = (self.frame + 1) % self.max_frames_in_flight;
        Ok(())
    }

    pub fn get_swapchain_framebuffer(&self) -> Handle<Framebuffer> {
        self.swapchain_framebuffers[self.image_index]
    }

    pub fn get_image_available_semaphore(&self) -> Handle<Semaphore> {
        self.image_availables[self.frame]
    }

    pub fn get_render_finished_semaphore(&self) -> Handle<Semaphore> {
        self.render_finisheds[self.frame]
    }

    pub fn get_in_flight_fence(&self) -> Handle<Fence> {
        self.in_flight_fences[self.frame]
    }

    pub fn cleanup_swapchain(&mut self) -> Result<()> {
        for fb in self.swapchain_framebuffers.iter() {
            self.backend.destroy_framebuffer(*fb)?;
        }
        self.backend.destroy_image(self.depth_image)?;
        self.backend.destroy_swapchain(self.swapchain)?;
        Ok(())
    }

    pub fn recreate_swapchain(&mut self, width: u32, height: u32) -> Result<()> {
        self.backend.device_wait_idle()?;

        self.cleanup_swapchain()?;

        self.swapchain = self.backend.create_swapchain(SwapchainCreateDesc {
            width: width,
            height: height,
            surface: self.surface,
            vsync: true,
            format: Format::B8G8R8A8_SRGB,
        })?;

        let (depth_image, depth_image_view) = create_depth(&mut self.backend, width, height)?;
        self.depth_image = depth_image;
        self.depth_image_view = depth_image_view;

        transition_image_layout(
            &mut self.backend,
            self.command_pool,
            self.graphics_queue,
            depth_image,
            ImageLayout::Undefined,
            ImageLayout::DepthStencilAttachmentOptimal,
            ImageAspectFlags::DEPTH,
        )?;

        self.swapchain_framebuffers.clear();
        for i in 0..self.max_frames_in_flight {
            let back_buffer = self.backend.get_swapchain_back_buffer(self.swapchain, i as usize)?;
            let view = self.backend.create_image_view(
                back_buffer,
                &ImageViewCreateDesc { ..ImageViewCreateDesc::default() },
            )?;
            let fb = self.backend.create_framebuffer(&FramebufferCreateDesc {
                render_pass: self.swapchain_render_pass,
                color_views: &[view],
                depth_stencil_view: Some(self.depth_image_view),
            })?;
            self.swapchain_framebuffers.push(fb);
        }

        Ok(())
    }

    pub fn destroy(&mut self) -> Result<()> {
        self.backend.destroy_image(self.depth_image)?;
        for i in 0..self.max_frames_in_flight {
            self.backend.destroy_fence(self.in_flight_fences[i])?;
            self.backend.destroy_semaphore(self.image_availables[i])?;
            self.backend.destroy_semaphore(self.render_finisheds[i])?;
        }

        self.cleanup_swapchain()?;

        self.backend.destroy_command_pool(self.command_pool).unwrap();

        self.backend.destroy_render_pass(self.swapchain_render_pass)?;
        self.backend.destroy_surface(self.surface)?;
        self.backend.destroy_device()?;
        Ok(())
    }
}

pub fn as_byte_slice_unchecked<T: Copy>(v: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * std::mem::size_of::<T>())
    }
}

pub fn create_depth(
    rhi: &mut Box<dyn RenderBackend>,
    width: u32,
    height: u32,
) -> Result<(Handle<Image>, Handle<ImageView>)> {
    let depth_format = rhi.get_supported_format_from_candidates(
        &[Format::D32_SFLOAT, Format::D24_UNORM_S8_UINT, Format::D32_SFLOAT_S8_UINT],
        ImageTiling::Optimal,
        FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )?;
    let depth_image =
        rhi.create_image(&ImageCreateDesc::new_depth("depth", depth_format, width, height))?;
    let depth_image_view = rhi.create_image_view(
        depth_image,
        &ImageViewCreateDesc::new_2d(None, ImageAspectFlags::DEPTH),
    )?;
    Ok((depth_image, depth_image_view))
}

pub fn copy_buffer(
    rhi: &mut Box<dyn RenderBackend>,
    command_pool: Handle<CommandPool>,
    queue: Handle<Queue>,
    src: Handle<Buffer>,
    dst: Handle<Buffer>,
    size: u64,
) -> Result<()> {
    let cb = begin_single_time_commands(rhi, command_pool)?;
    rhi.cmd_copy_buffer(cb, src, dst, &[BufferCopyRegion { size, ..Default::default() }])?;
    end_single_time_commands(rhi, cb, queue)?;
    Ok(())
}

pub fn upload_image_by_staging_buffer(
    rhi: &mut Box<dyn RenderBackend>,
    command_pool: Handle<CommandPool>,
    queue: Handle<Queue>,
    image: Handle<Image>,
    data: &[u8],
    width: u32,
    height: u32,
) -> Result<()> {
    let size = data.len() as u64;
    let staging_buffer = rhi.create_buffer(&BufferCreateDesc {
        name: "Staging Buffer",
        size: size,
        usage: BufferUsageFlags::TRANSFER_SRC,
        memory: MemoryLocation::CpuToGpu,
        initial_data: Some(data),
    })?;
    transition_image_layout(
        rhi,
        command_pool,
        queue,
        image,
        ImageLayout::Undefined,
        ImageLayout::TransferDstOptimal,
        ImageAspectFlags::COLOR,
    )?;
    copy_buffer_to_image(rhi, command_pool, queue, staging_buffer, image, width, height)?;
    transition_image_layout(
        rhi,
        command_pool,
        queue,
        image,
        ImageLayout::TransferDstOptimal,
        ImageLayout::ShaderReadOnlyOptimal,
        ImageAspectFlags::COLOR,
    )?;
    rhi.destroy_buffer(staging_buffer)?;
    Ok(())
}

pub fn upload_buffer_by_staging_buffer(
    rhi: &mut Box<dyn RenderBackend>,
    command_pool: Handle<CommandPool>,
    queue: Handle<Queue>,
    buffer: Handle<Buffer>,
    data: &[u8],
) -> Result<()> {
    let size = data.len() as u64;
    let staging_buffer = rhi.create_buffer(&BufferCreateDesc {
        name: "Staging Buffer",
        size: size,
        usage: BufferUsageFlags::TRANSFER_SRC,
        memory: MemoryLocation::CpuToGpu,
        initial_data: Some(data),
    })?;
    copy_buffer(rhi, command_pool, queue, staging_buffer, buffer, size)?;
    rhi.destroy_buffer(staging_buffer)?;
    Ok(())
}

pub fn copy_buffer_to_image(
    rhi: &mut Box<dyn RenderBackend>,
    command_pool: Handle<CommandPool>,
    queue: Handle<Queue>,
    src: Handle<Buffer>,
    dst: Handle<Image>,
    width: u32,
    height: u32,
) -> Result<()> {
    let cb = begin_single_time_commands(rhi, command_pool)?;
    rhi.cmd_copy_buffer_to_image(
        cb,
        src,
        dst,
        ImageLayout::TransferDstOptimal,
        &[BufferImageCopyRegion {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            aspect_mask: ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
            image_offset: [0, 0, 0],
            image_extent: [width, height, 1],
        }],
    )?;
    end_single_time_commands(rhi, cb, queue)?;
    Ok(())
}

pub fn compile_shader_glsl(
    backend: &mut Box<dyn RenderBackend>,
    name: &str,
    code: &str,
    stage: ShaderStageFlags,
    entry: &str,
) -> Result<Handle<Shader>> {
    let compiler = shaderc::Compiler::new().unwrap();
    backend.create_shader_module(&ShaderModuleCreation {
        name,
        code: compiler
            .compile_into_spirv(
                code,
                match stage {
                    ShaderStageFlags::VERTEX => shaderc::ShaderKind::Vertex,
                    ShaderStageFlags::FRAGMENT => shaderc::ShaderKind::Fragment,
                    ShaderStageFlags::COMPUTE => shaderc::ShaderKind::Compute,
                    _ => panic!("Unsupported shader stage"),
                },
                "shader.glsl",
                entry,
                None,
            )
            .unwrap()
            .as_binary(),
        stage,
        entry: entry,
    })
}

pub fn begin_single_time_commands(
    backend: &mut Box<dyn RenderBackend>,
    command_pool: Handle<CommandPool>,
) -> Result<Handle<CommandBuffer>> {
    let cb = backend.create_command_buffer(command_pool, CommandBufferLevel::Primary)?;
    backend.cmd_begin(cb, CommandBufferBeginDesc { one_time_submit: true })?;
    Ok(cb)
}

pub fn end_single_time_commands(
    backend: &mut Box<dyn RenderBackend>,
    cb: Handle<CommandBuffer>,
    queue: Handle<Queue>,
) -> Result<()> {
    backend.cmd_end(cb)?;
    backend.queue_submit(
        queue,
        &QueueSubmitDesc {
            wait_semaphore: None,
            wait_stage: None,
            command_buffer: &[cb],
            finish_semaphore: None,
            fence: None,
        },
    )?;
    // wait for the queue to finish executing the command buffer
    backend.queue_wait_idle(queue)?;
    backend.destroy_command_buffer(cb)?;
    Ok(())
}

pub fn transition_image_layout(
    backend: &mut Box<dyn RenderBackend>,
    command_pool: Handle<CommandPool>,
    queue: Handle<Queue>,
    image: Handle<Image>,
    old_layout: ImageLayout,
    new_layout: ImageLayout,
    aspect_mask: ImageAspectFlags,
) -> Result<()> {
    let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) =
        match (old_layout, new_layout) {
            (ImageLayout::Undefined, ImageLayout::TransferDstOptimal) => (
                AccessFlags::empty(),
                AccessFlags::TRANSFER_WRITE,
                PipelineStageFlags::TOP_OF_PIPE,
                PipelineStageFlags::TRANSFER,
            ),
            (ImageLayout::TransferDstOptimal, ImageLayout::ShaderReadOnlyOptimal) => (
                AccessFlags::TRANSFER_WRITE,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::TRANSFER,
                PipelineStageFlags::FRAGMENT_SHADER,
            ),
            (ImageLayout::Undefined, ImageLayout::DepthStencilAttachmentOptimal) => (
                AccessFlags::empty(),
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                    | AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                PipelineStageFlags::TOP_OF_PIPE,
                PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            ),
            _ => return Err(anyhow::anyhow!("Unsupported image layout transition!")),
        };

    let cb = begin_single_time_commands(backend, command_pool)?;
    backend.cmd_pipeline_barrier(
        cb,
        src_stage_mask,
        dst_stage_mask,
        &[ImageMemoryBarrier {
            image: image,
            old_layout: old_layout,
            new_layout: new_layout,
            src_queue_family_index: None,
            dst_queue_family_index: None,
            aspect_mask: aspect_mask,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
            src_access_mask,
            dst_access_mask,
        }],
    )?;
    end_single_time_commands(backend, cb, queue)?;
    Ok(())
}
