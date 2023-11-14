use anyhow::Ok;
use luxseed_rhi::{define::*, enums::*, pool::Handle};
use winit::window::Window;

use crate::render_system::RenderSystem;

pub struct App {
    pub sys: RenderSystem,

    pub vs: Handle<Shader>,
    pub fs: Handle<Shader>,
    pub pipeline: Handle<RasterPipeline>,
    pub command_pool: Handle<CommandPool>,
    pub command_buffers: Vec<Handle<CommandBuffer>>,
    pub framebuffers: Vec<Handle<Framebuffer>>,
    pub frame_buffer_view: Vec<Handle<TextureView>>,
}

impl App {
    pub fn create(window: &Window) -> anyhow::Result<Self> {
        let mut sys = RenderSystem::create(window)?;

        let vs = sys.compile_shader("hello", VERTEX_SHADER, ShaderStage::Vertex, "main")?;
        let fs = sys.compile_shader("hello", FRAGMENT_SHADER, ShaderStage::Fragment, "main")?;

        let pipeline = sys
            .rhi
            .create_raster_pipeline(
                sys.device,
                &RasterPipelineCreation::builder()
                    .add_shader_stages(&[vs, fs])
                    .add_blend_states(&[BlendState::default()])
                    .render_pass_output(sys.swapchain_output)
                    .raster_state(RasterState::default())
                    .depth_state(DepthState::default())
                    .build(),
            )
            .unwrap();
        let command_pool = sys.rhi.create_command_pool(sys.graphics_queue).unwrap();

        let image_count = sys.max_frames_in_flight;

        let mut command_buffers = Vec::new();
        let mut framebuffers = Vec::new();
        let mut frame_buffer_view = Vec::new();

        for i in 0..image_count {
            let back_buffer = sys.rhi.get_swapchain_back_buffer(sys.swapchain, i as usize)?;
            let view = sys.rhi.create_texture_view(
                sys.device,
                back_buffer,
                &TextureViewCreateDesc { ..TextureViewCreateDesc::default() },
            )?;
            let fb = sys.rhi.create_framebuffer(
                sys.device,
                &FramebufferCreateDesc {
                    render_pass: sys.swapchain_render_pass,
                    color_views: &[view],
                    depth_stencil_view: None,
                },
            )?;
            let cb = sys.rhi.create_command_buffer(command_pool, CommandBufferLevel::Primary)?;

            frame_buffer_view.push(view);
            framebuffers.push(fb);
            command_buffers.push(cb);
        }

        Ok(Self {
            sys,
            vs,
            fs,
            pipeline,
            command_pool,
            command_buffers,
            framebuffers,
            frame_buffer_view,
        })
    }

    pub fn render(&mut self, window: &Window) -> anyhow::Result<()> {
        self.sys.begin_frame()?;

        let image_index = self
            .sys
            .rhi
            .acquire_next_image(
                self.sys.swapchain,
                u64::MAX,
                self.sys.get_image_available_semaphore(),
                None,
            )
            .unwrap();
        let cb = self.command_buffers[self.sys.frame];
        self.sys.rhi.reset_command_buffer(cb, false)?;
        {
            self.sys.rhi.cmd_begin(cb)?;
            self.sys.rhi.cmd_begin_render_pass(
                cb,
                self.sys.swapchain_render_pass,
                self.framebuffers[image_index],
                Some(&[ClearColor::new([0.0, 0.0, 0.0, 1.0])]),
                None,
            )?;
            self.sys.rhi.cmd_bind_raster_pipeline(cb, self.pipeline)?;
            self.sys.rhi.cmd_set_viewport(
                cb,
                0.0,
                0.0,
                window.inner_size().width as f32,
                window.inner_size().height as f32,
                0.0,
                1.0,
            )?;
            self.sys.rhi.cmd_set_scissor(
                cb,
                0,
                0,
                window.inner_size().width,
                window.inner_size().height,
            )?;
            self.sys.rhi.cmd_draw(cb, 3, 1, 0, 0)?;
            self.sys.rhi.cmd_end_render_pass(cb)?;
            self.sys.rhi.cmd_end(cb)?;
        }

        self.sys.rhi.queue_submit(
            self.sys.graphics_queue,
            &QueueSubmitDesc {
                wait_semaphore: &[self.sys.get_image_available_semaphore()],
                wait_stage: &[PipelineStage::ColorAttachmentOutput],
                command_buffer: &[self.command_buffers[self.sys.frame]],
                finish_semaphore: &[self.sys.get_render_finished_semaphore()],
                fence: Some(self.sys.get_in_flight_fence()),
            },
        )?;

        self.sys.rhi.queue_present(
            self.sys.graphics_queue,
            &QueuePresentDesc {
                swapchain: self.sys.swapchain,
                image_index: image_index as u32,
                wait_semaphores: &[self.sys.get_render_finished_semaphore()],
            },
        )?;

        self.sys.end_frame()?;
        Ok(())
    }

    pub fn destroy(&mut self) {
        self.sys.rhi.wait_idle(self.sys.device).unwrap();

        self.sys.rhi.destroy_command_pool(self.command_pool).unwrap();
        self.sys.rhi.destroy_shader_module(self.vs).unwrap();
        self.sys.rhi.destroy_shader_module(self.fs).unwrap();
        for fb in self.framebuffers.iter() {
            self.sys.rhi.destroy_framebuffer(*fb).unwrap();
        }
        for fbv in self.frame_buffer_view.iter() {
            self.sys.rhi.destroy_texture_view(*fbv).unwrap();
        }
        self.sys.rhi.destroy_raster_pipeline(self.pipeline).unwrap();

        self.sys.destroy().unwrap();
    }
}

const VERTEX_SHADER: &str = "
#version 450

layout(location = 0) out vec3 fragColor;

vec2 positions[3] = vec2[](
    vec2(0.0, -0.5),
    vec2(0.5, 0.5),
    vec2(-0.5, 0.5)
);

vec3 colors[3] = vec3[](
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0)
);

void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
    fragColor = colors[gl_VertexIndex];
}
";

const FRAGMENT_SHADER: &str = "
#version 450

layout(location = 0) in vec3 fragColor;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(fragColor, 1.0);
}
";
