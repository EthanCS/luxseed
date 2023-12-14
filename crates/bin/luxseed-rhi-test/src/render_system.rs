extern crate shaderc;

use anyhow::{self, Ok, Result};
use luxseed_rhi::{define::*, enums::*, flag::*, pool::Handle, rhi_create, RHI};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::window::Window;

pub struct RenderSystem {
    pub rhi: Box<dyn RHI>,
    pub device: Handle<Device>,
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
}

impl RenderSystem {
    pub fn create(window: &Window) -> Result<Self> {
        let mut rhi = rhi_create(
            BackendType::Vulkan,
            RHICreation {
                app_name: "Luxseed Vulkan - Hello World",
                app_version: 0,
                enable_debugging: true,
                raw_display_handle: window.raw_display_handle(),
            },
        )?;
        let format = Format::B8G8R8A8_SRGB;
        let device = rhi.create_device(0)?;
        let surface = rhi.create_surface(SurfaceCreateDesc {
            raw_display_handle: window.raw_display_handle(),
            raw_window_handle: window.raw_window_handle(),
        })?;
        let swapchain = rhi.create_swapchain(
            device,
            SwapchainCreation {
                width: window.inner_size().width,
                height: window.inner_size().height,
                surface: surface,
                vsync: true,
                format,
            },
        )?;
        let max_frames_in_flight = rhi.get_swapchain_image_count(swapchain)? as usize;
        let graphics_queue = rhi.get_queue(device, QueueType::Graphics)?;
        let swapchain_output = RenderPassOutput::builder()
            .add_color(
                format,
                ImageLayout::PresentSrcKhr,
                RenderTargetLoadAction::Clear,
                SampleCount::Sample1,
            )
            .build();
        let swapchain_render_pass = rhi.create_render_pass(device, &swapchain_output)?;

        let mut in_flight_fences = Vec::new();
        let mut image_availables = Vec::new();
        let mut render_finisheds = Vec::new();
        let mut swapchain_framebuffers = Vec::new();

        for i in 0..max_frames_in_flight {
            in_flight_fences.push(rhi.create_fence(device, true)?);
            image_availables.push(rhi.create_semaphore(device)?);
            render_finisheds.push(rhi.create_semaphore(device)?);

            let back_buffer = rhi.get_swapchain_back_buffer(swapchain, i as usize)?;
            let view = rhi.create_image_view(
                device,
                back_buffer,
                &TextureViewCreateDesc { ..TextureViewCreateDesc::default() },
            )?;
            let fb = rhi.create_framebuffer(
                device,
                &FramebufferCreateDesc {
                    render_pass: swapchain_render_pass,
                    color_views: &[view],
                    depth_stencil_view: None,
                },
            )?;
            swapchain_framebuffers.push(fb);
        }

        let command_pool = rhi.create_command_pool(graphics_queue).unwrap();

        Ok(Self {
            rhi,
            device,
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
        })
    }

    pub fn begin_frame(&mut self, width: u32, height: u32) -> Result<bool> {
        self.rhi.wait_for_fences(&[self.get_in_flight_fence()], true, u64::MAX)?;

        let image_index = self.rhi.acquire_swapchain_next_image(
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
        self.rhi.reset_fences(&[self.get_in_flight_fence()])?;

        Ok(true)
    }

    pub fn end_frame(&mut self, recreate_swapchain: bool, width: u32, height: u32) -> Result<()> {
        let suboptimal = self.rhi.queue_present(
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

    pub fn upload_buffer_by_staging_buffer(
        &mut self,
        buffer: Handle<Buffer>,
        data: &[u8],
    ) -> Result<()> {
        let size = data.len() as u64;
        let staging_buffer = self.rhi.create_buffer(
            self.device,
            &BufferCreateDesc {
                name: "Staging Buffer",
                size: size,
                usage: BufferUsageFlags::TRANSFER_SRC,
                memory: MemoryLocation::CpuToGpu,
                initial_data: Some(data),
            },
        )?;
        self.copy_buffer(staging_buffer, buffer, size)?;
        self.rhi.destroy_buffer(staging_buffer)?;
        Ok(())
    }

    pub fn begin_single_time_commands(&mut self) -> Result<Handle<CommandBuffer>> {
        let cb = self.rhi.create_command_buffer(self.command_pool, CommandBufferLevel::Primary)?;
        self.rhi.cmd_begin(cb, CommandBufferBeginDesc { one_time_submit: true })?;
        Ok(cb)
    }

    pub fn end_single_time_commands(&mut self, cb: Handle<CommandBuffer>) -> Result<()> {
        self.rhi.cmd_end(cb)?;
        self.rhi.queue_submit(
            self.graphics_queue,
            &QueueSubmitDesc {
                wait_semaphore: None,
                wait_stage: None,
                command_buffer: &[cb],
                finish_semaphore: None,
                fence: None,
            },
        )?;
        // wait for the queue to finish executing the command buffer
        self.rhi.queue_wait_idle(self.graphics_queue)?;
        self.rhi.destroy_command_buffer(cb)?;
        Ok(())
    }

    pub fn copy_buffer(
        &mut self,
        src: Handle<Buffer>,
        dst: Handle<Buffer>,
        size: u64,
    ) -> Result<()> {
        let cb = self.begin_single_time_commands()?;
        self.rhi.cmd_copy_buffer(
            cb,
            src,
            dst,
            &[BufferCopyRegion { size, ..Default::default() }],
        )?;
        self.end_single_time_commands(cb)?;
        Ok(())
    }

    pub fn compile_shader(
        &mut self,
        name: &str,
        code: &str,
        stage: ShaderStageFlags,
        entry: &str,
    ) -> Result<Handle<Shader>> {
        let compiler = shaderc::Compiler::new().unwrap();
        self.rhi.create_shader_module(
            self.device,
            &ShaderModuleCreation {
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
            },
        )
    }

    pub fn cleanup_swapchain(&mut self) -> Result<()> {
        for fb in self.swapchain_framebuffers.iter() {
            self.rhi.destroy_framebuffer(*fb)?;
        }
        self.rhi.destroy_swapchain(self.swapchain)?;
        Ok(())
    }

    pub fn recreate_swapchain(&mut self, width: u32, height: u32) -> Result<()> {
        self.rhi.wait_idle(self.device)?;

        self.cleanup_swapchain()?;

        self.swapchain = self.rhi.create_swapchain(
            self.device,
            SwapchainCreation {
                width: width,
                height: height,
                surface: self.surface,
                vsync: true,
                format: Format::B8G8R8A8_SRGB,
            },
        )?;

        self.swapchain_framebuffers.clear();
        for i in 0..self.max_frames_in_flight {
            let back_buffer = self.rhi.get_swapchain_back_buffer(self.swapchain, i as usize)?;
            let view = self.rhi.create_image_view(
                self.device,
                back_buffer,
                &TextureViewCreateDesc { ..TextureViewCreateDesc::default() },
            )?;
            let fb = self.rhi.create_framebuffer(
                self.device,
                &FramebufferCreateDesc {
                    render_pass: self.swapchain_render_pass,
                    color_views: &[view],
                    depth_stencil_view: None,
                },
            )?;
            self.swapchain_framebuffers.push(fb);
        }

        Ok(())
    }

    pub fn destroy(&mut self) -> Result<()> {
        for i in 0..self.max_frames_in_flight {
            self.rhi.destroy_fence(self.in_flight_fences[i])?;
            self.rhi.destroy_semaphore(self.image_availables[i])?;
            self.rhi.destroy_semaphore(self.render_finisheds[i])?;
        }

        self.cleanup_swapchain()?;

        self.rhi.destroy_command_pool(self.command_pool).unwrap();

        self.rhi.destroy_render_pass(self.swapchain_render_pass)?;
        self.rhi.destroy_surface(self.surface)?;
        self.rhi.destroy_device(self.device)?;
        Ok(())
    }
}
