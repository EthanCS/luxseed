mod render_system;

use glam::{vec2, vec3, Mat4, Vec2, Vec3};
use image::{io::Reader as ImageReader, EncodableLayout};
use luxseed_render_backend::{define::*, enums::*, flag::*};
use luxseed_utility::pool::Handle;
use render_system::*;
use std::{fs, mem::size_of};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

const WIDTH: u32 = 1600;
const HEIGHT: u32 = 900;

fn main() -> anyhow::Result<()> {
    // Window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Luxseed RHI Test App")
        .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)?;

    // App
    let mut app = App::create(&window)?;
    let mut destroying = false;
    let mut minimized = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            // Render a frame if our Vulkan app is not being destroyed.
            Event::MainEventsCleared if !destroying && !minimized => app.render(&window).unwrap(),
            // Mark the window as having been resized.
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                if size.width == 0 || size.height == 0 {
                    minimized = true;
                } else {
                    minimized = false;
                    app.resize = true;
                }
            }
            // Destroy our Vulkan app.
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                destroying = true;
                *control_flow = ControlFlow::Exit;
                app.destroy();
            }
            _ => {}
        }
    });
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: Vec3,
    pub color: Vec3,
    pub tex_coord: Vec2,
}

impl Vertex {
    const fn new(pos: Vec3, color: Vec3, tex_coord: Vec2) -> Self {
        Self { pos, color, tex_coord }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UniformBufferObject {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
}

pub struct App {
    pub start: std::time::Instant,
    pub sys: RenderSystem,
    pub resize: bool,
    pub vs: Handle<Shader>,
    pub fs: Handle<Shader>,

    pub pipeline_layout: Handle<PipelineLayout>,
    pub pipeline: Handle<RasterPipeline>,
    pub descriptor_set_layout: Handle<DescriptorSetLayout>,
    pub descriptor_pool: Handle<DescriptorPool>,

    pub command_buffers: Vec<Handle<CommandBuffer>>,
    pub uniform_buffers: Vec<Handle<Buffer>>,
    pub descriptor_sets: Vec<Handle<DescriptorSet>>,

    pub vertex_buffer: Handle<Buffer>,
    pub vertices: Vec<Vertex>,
    pub index_buffer: Handle<Buffer>,
    pub indices: Vec<u16>,

    pub image: Handle<Image>,
    pub image_view: Handle<ImageView>,
    pub sampler: Handle<Sampler>,
}

impl App {
    pub fn create(window: &Window) -> anyhow::Result<Self> {
        let mut sys = RenderSystem::create(window)?;

        let vs = compile_shader_glsl(
            &mut sys.backend,
            "hello",
            &fs::read_to_string("assets/luxseed-render-backend-test/depth_buffer.vert")
                .expect("Should have been able to read the file"),
            ShaderStageFlags::VERTEX,
            "main",
        )?;
        let fs = compile_shader_glsl(
            &mut sys.backend,
            "hello",
            &fs::read_to_string("assets/luxseed-render-backend-test/hello_world.frag")
                .expect("Should have been able to read the file"),
            ShaderStageFlags::FRAGMENT,
            "main",
        )?;

        // Load image
        let img = ImageReader::open("assets/luxseed-render-backend-test/lue.jpg")?.decode()?;
        let image = sys.backend.create_image(&ImageCreateDesc::new_2d(
            "lue.jpg",
            Format::R8G8B8A8_SRGB,
            img.width(),
            img.height(),
        ))?;
        upload_image_by_staging_buffer(
            &mut sys.backend,
            sys.command_pool,
            sys.graphics_queue,
            image,
            img.to_rgba8().as_bytes(),
            img.width(),
            img.height(),
        )?;

        // Image view
        let image_view = sys.backend.create_image_view(image, &ImageViewCreateDesc::default())?;

        // Sampler
        let sampler = sys.backend.create_sampler(&SamplerCreateDesc {
            mag_filter: FilterType::Linear,
            min_filter: FilterType::Linear,
            mipmap_mode: SamplerMipmapMode::Linear,
            address_mode_u: SamplerAddressMode::Repeat,
            address_mode_v: SamplerAddressMode::Repeat,
            address_mode_w: SamplerAddressMode::Repeat,
            mip_lod_bias: 0.0,
            compare_op: None,
            max_anisotropy: None,
        })?;

        // Vertex buffer
        let vertices = vec![
            Vertex::new(vec3(-0.5, -0.5, 0.0), vec3(1.0, 0.0, 0.0), vec2(1.0, 0.0)),
            Vertex::new(vec3(0.5, -0.5, 0.0), vec3(0.0, 1.0, 0.0), vec2(0.0, 0.0)),
            Vertex::new(vec3(0.5, 0.5, 0.0), vec3(0.0, 0.0, 1.0), vec2(0.0, 1.0)),
            Vertex::new(vec3(-0.5, 0.5, 0.0), vec3(1.0, 1.0, 1.0), vec2(1.0, 1.0)),
            Vertex::new(vec3(-0.5, -0.5, -0.5), vec3(1.0, 0.0, 0.0), vec2(1.0, 0.0)),
            Vertex::new(vec3(0.5, -0.5, -0.5), vec3(0.0, 1.0, 0.0), vec2(0.0, 0.0)),
            Vertex::new(vec3(0.5, 0.5, -0.5), vec3(0.0, 0.0, 1.0), vec2(0.0, 1.0)),
            Vertex::new(vec3(-0.5, 0.5, -0.5), vec3(1.0, 1.0, 1.0), vec2(1.0, 1.0)),
        ];
        let vertex_buffer = sys.backend.create_buffer(&BufferCreateDesc {
            name: "Triangle_Vertex",
            size: (vertices.len() * std::mem::size_of::<Vertex>()) as u64,
            usage: BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER,
            memory: MemoryLocation::GpuOnly,
            initial_data: None,
        })?;
        upload_buffer_by_staging_buffer(
            &mut sys.backend,
            sys.command_pool,
            sys.graphics_queue,
            vertex_buffer,
            as_byte_slice_unchecked(&vertices),
        )?;

        // Index buffer
        let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4];
        let index_buffer = sys.backend.create_buffer(&BufferCreateDesc {
            name: "Triangle_Index",
            size: (indices.len() * std::mem::size_of::<u16>()) as u64,
            usage: BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::INDEX_BUFFER,
            memory: MemoryLocation::GpuOnly,
            initial_data: None,
        })?;
        upload_buffer_by_staging_buffer(
            &mut sys.backend,
            sys.command_pool,
            sys.graphics_queue,
            index_buffer,
            as_byte_slice_unchecked(&indices),
        )?;

        // UBOs
        let mut uniform_buffers = Vec::new();
        for _ in 0..sys.max_frames_in_flight {
            let ub = sys.backend.create_buffer(&BufferCreateDesc {
                name: "Triangle_UBO",
                size: std::mem::size_of::<UniformBufferObject>() as u64,
                usage: BufferUsageFlags::UNIFORM_BUFFER,
                memory: MemoryLocation::CpuToGpu,
                initial_data: None,
            })?;
            uniform_buffers.push(ub);
        }

        // Descriptor set layout
        let descriptor_set_layout = sys.backend.create_descriptor_set_layout(
            &DescriptorSetLayoutCreateDesc::new()
                .add_binding_info(DescriptorBindingInfo {
                    index: 0,
                    type_: DescriptorType::UniformBuffer,
                    count: 1,
                    stage_flags: ShaderStageFlags::VERTEX,
                })
                .add_binding_info(DescriptorBindingInfo {
                    index: 1,
                    type_: DescriptorType::CombinedImageSampler,
                    count: 1,
                    stage_flags: ShaderStageFlags::FRAGMENT,
                }),
        )?;

        // Descriptor pool
        let descriptor_pool = sys.backend.create_descriptor_pool(&DescriptorPoolCreateDesc {
            max_sets: sys.max_frames_in_flight as u32,
            pool_sizes: &[
                DescriptorPoolSize {
                    descriptor_type: DescriptorType::UniformBuffer,
                    descriptor_count: sys.max_frames_in_flight as u32,
                },
                DescriptorPoolSize {
                    descriptor_type: DescriptorType::CombinedImageSampler,
                    descriptor_count: sys.max_frames_in_flight as u32,
                },
            ],
        })?;

        // Pipeline layout
        let pipeline_layout = sys.backend.create_pipeline_layout(&PipelineLayoutCreateDesc {
            descriptor_set_layouts: &[descriptor_set_layout],
        })?;

        // Pipeline
        let pipeline = sys
            .backend
            .create_raster_pipeline(&RasterPipelineCreateDesc {
                vertex_input_bindings: Some(&[VertexInputBinding {
                    stride: std::mem::size_of::<Vertex>(),
                    input_rate: VertexInputRate::Vertex,
                    attributes: &[
                        VertexInputAttribute { offset: 0, format: Format::R32G32B32_SFLOAT },
                        VertexInputAttribute {
                            offset: size_of::<Vec3>(),
                            format: Format::R32G32B32_SFLOAT,
                        },
                        VertexInputAttribute {
                            offset: (size_of::<Vec3>() + size_of::<Vec3>()),
                            format: Format::R32G32_SFLOAT,
                        },
                    ],
                }]),
                shader_stages: &[vs, fs],
                render_pass_output: sys.swapchain_output,
                blend_states: &[BlendState::default()],
                raster_state: RasterState {
                    front_face: FrontFace::CounterClockwise,
                    ..Default::default()
                },
                depth_state: DepthState::default(),
                pipeline_layout,
            })
            .unwrap();

        let mut command_buffers = Vec::new();
        let mut descriptor_sets = Vec::new();
        for i in 0..sys.max_frames_in_flight {
            command_buffers.push(
                sys.backend.create_command_buffer(sys.command_pool, CommandBufferLevel::Primary)?,
            );
            let descriptor_set = sys.backend.create_descriptor_set(
                &DescriptorSetCreateDesc::new(descriptor_pool, descriptor_set_layout)
                    .bind_uniform_buffer(0, uniform_buffers[i])
                    .bind_combined_image_sampler(1, image_view, sampler),
            )?;
            descriptor_sets.push(descriptor_set);
        }

        Ok(Self {
            start: std::time::Instant::now(),
            sys,
            vs,
            fs,
            pipeline,
            pipeline_layout,
            descriptor_set_layout,
            descriptor_pool,
            descriptor_sets,
            command_buffers,
            uniform_buffers,
            resize: false,
            vertex_buffer,
            vertices,
            index_buffer,
            indices,
            image,
            image_view,
            sampler,
        })
    }

    fn update_uniform_buffer(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        let time = self.start.elapsed().as_secs_f32();

        let mut ubo = UniformBufferObject {
            model: Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), time * 90.0_f32.to_radians()),
            view: Mat4::look_at_rh(vec3(2.0, 2.0, 2.0), vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0)),
            proj: Mat4::perspective_rh(
                45.0_f32.to_radians(),
                width as f32 / height as f32,
                0.1,
                10.0,
            ),
        };
        ubo.proj.col_mut(1)[1] *= -1.0;

        let ub = self.uniform_buffers[self.sys.frame];
        self.sys
            .backend
            .get_buffer_mapped_slice_mut(ub)?
            .copy_from_slice(as_byte_slice_unchecked(&[ubo]));
        Ok(())
    }

    pub fn render(&mut self, window: &Window) -> anyhow::Result<()> {
        let width = window.inner_size().width;
        let height = window.inner_size().height;

        if self.sys.begin_frame(width, height)? {
            self.update_uniform_buffer(width, height)?;

            let cb = self.command_buffers[self.sys.frame];
            self.sys.backend.reset_command_buffer(cb, false)?;
            self.sys.backend.cmd_begin(cb, CommandBufferBeginDesc::default())?;
            let rp = self.sys.swapchain_render_pass;
            let fb = self.sys.get_swapchain_framebuffer();
            let cv = ClearColor::new([0.0, 0.0, 0.0, 1.0]);
            let cd = ClearDepthStencil { depth: 1.0, stencil: 0 };
            self.sys.backend.cmd_begin_render_pass(cb, rp, fb, Some(&[cv]), Some(cd))?;
            self.sys.backend.cmd_bind_raster_pipeline(cb, self.pipeline)?;
            self.sys.backend.cmd_set_viewport(
                cb,
                0.0,
                0.0,
                width as f32,
                height as f32,
                0.0,
                1.0,
            )?;
            self.sys.backend.cmd_set_scissor(cb, 0, 0, width, height)?;
            self.sys.backend.cmd_bind_vertex_buffers(cb, 0, &[self.vertex_buffer], &[0])?;
            self.sys.backend.cmd_bind_index_buffer(cb, self.index_buffer, 0, IndexType::U16)?;
            self.sys.backend.cmd_bind_descriptor_sets(
                cb,
                PipelineBindPoint::Graphics,
                self.pipeline_layout,
                0,
                &[self.descriptor_sets[self.sys.frame]],
                &[],
            )?;
            self.sys.backend.cmd_draw_indexed(cb, self.indices.len() as u32, 1, 0, 0, 0)?;
            self.sys.backend.cmd_end_render_pass(cb)?;
            self.sys.backend.cmd_end(cb)?;
            self.sys.backend.queue_submit(
                self.sys.graphics_queue,
                &QueueSubmitDesc {
                    wait_semaphore: Some(&[self.sys.get_image_available_semaphore()]),
                    wait_stage: Some(&[PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT]),
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
        self.sys.backend.device_wait_idle().unwrap();

        self.sys.backend.destroy_image(self.image).unwrap();
        self.sys.backend.destroy_sampler(self.sampler).unwrap();
        self.sys.backend.destroy_buffer(self.vertex_buffer).unwrap();
        self.sys.backend.destroy_buffer(self.index_buffer).unwrap();
        for ub in self.uniform_buffers.iter() {
            self.sys.backend.destroy_buffer(*ub).unwrap();
        }

        self.sys.backend.destroy_shader_module(self.vs).unwrap();
        self.sys.backend.destroy_shader_module(self.fs).unwrap();
        self.sys.backend.destroy_descriptor_set_layout(self.descriptor_set_layout).unwrap();

        self.sys.backend.destroy_descriptor_pool(self.descriptor_pool).unwrap();
        self.sys.backend.destroy_pipeline_layout(self.pipeline_layout).unwrap();
        self.sys.backend.destroy_raster_pipeline(self.pipeline).unwrap();

        self.sys.destroy().unwrap();
    }
}
