use anyhow::Ok;
use luxseed_rhi::{define::*, enums::*, pool::Handle};
use winit::window::Window;

use crate::render_system::RenderSystem;

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

pub struct App {
    pub sys: RenderSystem,
    pub resize: bool,
    pub vs: Handle<Shader>,
    pub fs: Handle<Shader>,

    pub pipeline: Handle<RasterPipeline>,
    pub command_buffers: Vec<Handle<CommandBuffer>>,

    pub vertex_buffer: Handle<Buffer>,
    pub vertices: Vec<Vertex>,
    pub index_buffer: Handle<Buffer>,
    pub indices: Vec<u16>,
}

fn as_byte_slice_unchecked<T: Copy>(v: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * std::mem::size_of::<T>())
    }
}

impl App {
    pub fn create(window: &Window) -> anyhow::Result<Self> {
        let mut sys = RenderSystem::create(window)?;

        let vs = sys.compile_shader("hello", VERTEX_SHADER, ShaderStage::Vertex, "main")?;
        let fs = sys.compile_shader("hello", FRAGMENT_SHADER, ShaderStage::Fragment, "main")?;

        // Vertex buffer
        let vertices = vec![
            Vertex { position: [-0.5, -0.5], color: [1.0, 0.0, 0.0] },
            Vertex { position: [0.5, -0.5], color: [0.0, 1.0, 0.0] },
            Vertex { position: [0.5, 0.5], color: [0.0, 0.0, 1.0] },
            Vertex { position: [-0.5, 0.5], color: [1.0, 1.0, 1.0] },
        ];
        let vertex_buffer = sys.rhi.create_buffer(
            sys.device,
            &BufferCreateDesc {
                name: "Triangle_Vertex",
                size: vertices.len() * std::mem::size_of::<Vertex>(),
                usage: BufferUsage::TRANSFER_DST | BufferUsage::VERTEX_BUFFER,
                memory: MemoryLocation::GpuOnly,
                initial_data: None,
            },
        )?;
        sys.upload_buffer_by_staging_buffer(vertex_buffer, as_byte_slice_unchecked(&vertices))?;

        // Index buffer
        let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];
        let index_buffer = sys.rhi.create_buffer(
            sys.device,
            &BufferCreateDesc {
                name: "Triangle_Index",
                size: indices.len() * std::mem::size_of::<u16>(),
                usage: BufferUsage::TRANSFER_DST | BufferUsage::INDEX_BUFFER,
                memory: MemoryLocation::GpuOnly,
                initial_data: None,
            },
        )?;
        sys.upload_buffer_by_staging_buffer(index_buffer, as_byte_slice_unchecked(&indices))?;

        // Pipeline
        let pipeline = sys
            .rhi
            .create_raster_pipeline(
                sys.device,
                &RasterPipelineCreateDesc {
                    vertex_input_bindings: Some(&[VertexInputBinding {
                        stride: std::mem::size_of::<Vertex>(),
                        input_rate: VertexInputRate::Vertex,
                        attributes: &[
                            VertexInputAttribute { offset: 0, format: Format::R32G32_SFLOAT },
                            VertexInputAttribute { offset: 8, format: Format::R32G32B32_SFLOAT },
                        ],
                    }]),
                    shader_stages: &[vs, fs],
                    render_pass_output: sys.swapchain_output,
                    blend_states: &[BlendState::default()],
                    raster_state: RasterState::default(),
                    depth_state: DepthState::default(),
                },
            )
            .unwrap();

        let mut command_buffers = Vec::new();
        for _ in 0..sys.max_frames_in_flight {
            let cb =
                sys.rhi.create_command_buffer(sys.command_pool, CommandBufferLevel::Primary)?;
            command_buffers.push(cb);
        }

        Ok(Self {
            sys,
            vs,
            fs,
            pipeline,
            command_buffers,
            resize: false,
            vertex_buffer,
            vertices,
            index_buffer,
            indices,
        })
    }

    pub fn render(&mut self, window: &Window) -> anyhow::Result<()> {
        let width = window.inner_size().width;
        let height = window.inner_size().height;

        if self.sys.begin_frame(width, height)? {
            let cb = self.command_buffers[self.sys.frame];
            self.sys.rhi.reset_command_buffer(cb, false)?;
            self.sys.rhi.cmd_begin(cb)?;
            let rp = self.sys.swapchain_render_pass;
            let fb = self.sys.get_swapchain_framebuffer();
            let cv = ClearColor::new([0.0, 0.0, 0.0, 1.0]);
            self.sys.rhi.cmd_begin_render_pass(cb, rp, fb, Some(&[cv]), None)?;
            self.sys.rhi.cmd_bind_raster_pipeline(cb, self.pipeline)?;
            self.sys.rhi.cmd_set_viewport(cb, 0.0, 0.0, width as f32, height as f32, 0.0, 1.0)?;
            self.sys.rhi.cmd_set_scissor(cb, 0, 0, width, height)?;
            self.sys.rhi.cmd_bind_vertex_buffers(cb, 0, &[self.vertex_buffer], &[0])?;
            self.sys.rhi.cmd_bind_index_buffer(cb, self.index_buffer, 0, IndexType::U16)?;
            self.sys.rhi.cmd_draw_indexed(cb, self.indices.len() as u32, 1, 0, 0, 0)?;
            self.sys.rhi.cmd_end_render_pass(cb)?;
            self.sys.rhi.cmd_end(cb)?;
            self.sys.rhi.queue_submit(
                self.sys.graphics_queue,
                &QueueSubmitDesc {
                    wait_semaphore: Some(&[self.sys.get_image_available_semaphore()]),
                    wait_stage: Some(&[PipelineStage::ColorAttachmentOutput]),
                    command_buffer: &[cb],
                    finish_semaphore: Some(&[self.sys.get_render_finished_semaphore()]),
                    fence: Some(self.sys.get_in_flight_fence()),
                },
            )?;
            self.sys.end_frame(self.resize, width, height)?;
            self.resize = false;
        }

        Ok(())
    }

    pub fn destroy(&mut self) {
        self.sys.rhi.wait_idle(self.sys.device).unwrap();

        self.sys.rhi.destroy_buffer(self.vertex_buffer).unwrap();
        self.sys.rhi.destroy_buffer(self.index_buffer).unwrap();

        self.sys.rhi.destroy_shader_module(self.vs).unwrap();
        self.sys.rhi.destroy_shader_module(self.fs).unwrap();
        self.sys.rhi.destroy_raster_pipeline(self.pipeline).unwrap();

        self.sys.destroy().unwrap();
    }
}

const VERTEX_SHADER: &str = "
#version 450

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 fragColor;

void main() {
    gl_Position = vec4(inPosition, 0.0, 1.0);
    fragColor = inColor;
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
