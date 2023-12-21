use anyhow::Ok;
use glam::{vec2, vec3, Mat4, Vec2, Vec3};
use image::{io::Reader as ImageReader, EncodableLayout};
use luxseed_rhi::{define::*, enums::*, flag::*, pool::Handle};
use std::{fs, mem::size_of};
use winit::window::Window;

use crate::render_system::RenderSystem;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: Vec2,
    pub color: Vec3,
    pub tex_coord: Vec2,
}

impl Vertex {
    const fn new(pos: Vec2, color: Vec3, tex_coord: Vec2) -> Self {
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

fn as_byte_slice_unchecked<T: Copy>(v: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * std::mem::size_of::<T>())
    }
}

impl App {
    pub fn create(window: &Window) -> anyhow::Result<Self> {
        let mut sys = RenderSystem::create(window)?;

        let vs = sys.compile_shader(
            "hello",
            &fs::read_to_string("assets/luxseed-rhi-test/hello_world.vert")
                .expect("Should have been able to read the file"),
            ShaderStageFlags::VERTEX,
            "main",
        )?;
        let fs = sys.compile_shader(
            "hello",
            &fs::read_to_string("assets/luxseed-rhi-test/hello_world.frag")
                .expect("Should have been able to read the file"),
            ShaderStageFlags::FRAGMENT,
            "main",
        )?;

        // Load image
        let img = ImageReader::open("assets/luxseed-rhi-test/lue.jpg")?.decode()?;
        let image = sys.rhi.create_image(&ImageCreateDesc::new_2d(
            "lue.jpg",
            Format::R8G8B8A8_SRGB,
            img.width(),
            img.height(),
        ))?;
        sys.upload_image_by_staging_buffer(
            image,
            img.to_rgba8().as_bytes(),
            img.width(),
            img.height(),
        )?;

        // Image view
        let image_view = sys.rhi.create_image_view(image, &ImageViewCreateDesc::default())?;

        // Sampler
        let sampler = sys.rhi.create_sampler(&SamplerCreateDesc {
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
            Vertex::new(vec2(-0.5, -0.5), vec3(1.0, 0.0, 0.0), vec2(0.0, 0.0)),
            Vertex::new(vec2(0.5, -0.5), vec3(0.0, 1.0, 0.0), vec2(1.0, 0.0)),
            Vertex::new(vec2(0.5, 0.5), vec3(0.0, 0.0, 1.0), vec2(1.0, 1.0)),
            Vertex::new(vec2(-0.5, 0.5), vec3(1.0, 1.0, 1.0), vec2(0.0, 1.0)),
        ];
        let vertex_buffer = sys.rhi.create_buffer(&BufferCreateDesc {
            name: "Triangle_Vertex",
            size: (vertices.len() * std::mem::size_of::<Vertex>()) as u64,
            usage: BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER,
            memory: MemoryLocation::GpuOnly,
            initial_data: None,
        })?;
        sys.upload_buffer_by_staging_buffer(vertex_buffer, as_byte_slice_unchecked(&vertices))?;

        // Index buffer
        let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];
        let index_buffer = sys.rhi.create_buffer(&BufferCreateDesc {
            name: "Triangle_Index",
            size: (indices.len() * std::mem::size_of::<u16>()) as u64,
            usage: BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::INDEX_BUFFER,
            memory: MemoryLocation::GpuOnly,
            initial_data: None,
        })?;
        sys.upload_buffer_by_staging_buffer(index_buffer, as_byte_slice_unchecked(&indices))?;

        // UBOs
        let mut uniform_buffers = Vec::new();
        for _ in 0..sys.max_frames_in_flight {
            let ub = sys.rhi.create_buffer(&BufferCreateDesc {
                name: "Triangle_UBO",
                size: std::mem::size_of::<UniformBufferObject>() as u64,
                usage: BufferUsageFlags::UNIFORM_BUFFER,
                memory: MemoryLocation::CpuToGpu,
                initial_data: None,
            })?;
            uniform_buffers.push(ub);
        }

        // Descriptor set layout
        let descriptor_set_layout = sys.rhi.create_descriptor_set_layout(
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
        let descriptor_pool = sys.rhi.create_descriptor_pool(&DescriptorPoolCreateDesc {
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
        let pipeline_layout = sys.rhi.create_pipeline_layout(&PipelineLayoutCreateDesc {
            descriptor_set_layouts: &[descriptor_set_layout],
        })?;

        // Pipeline
        let pipeline = sys
            .rhi
            .create_raster_pipeline(&RasterPipelineCreateDesc {
                vertex_input_bindings: Some(&[VertexInputBinding {
                    stride: std::mem::size_of::<Vertex>(),
                    input_rate: VertexInputRate::Vertex,
                    attributes: &[
                        VertexInputAttribute { offset: 0, format: Format::R32G32_SFLOAT },
                        VertexInputAttribute {
                            offset: size_of::<Vec2>(),
                            format: Format::R32G32B32_SFLOAT,
                        },
                        VertexInputAttribute {
                            offset: (size_of::<Vec2>() + size_of::<Vec3>()),
                            format: Format::R32G32_SFLOAT,
                        },
                    ],
                }]),
                shader_stages: &[vs, fs],
                render_pass_output: sys.swapchain_output,
                blend_states: &[BlendState::default()],
                raster_state: RasterState::default(),
                depth_state: DepthState::default(),
                pipeline_layout,
            })
            .unwrap();

        let mut command_buffers = Vec::new();
        let mut descriptor_sets = Vec::new();
        for i in 0..sys.max_frames_in_flight {
            command_buffers.push(
                sys.rhi.create_command_buffer(sys.command_pool, CommandBufferLevel::Primary)?,
            );
            let descriptor_set = sys.rhi.create_descriptor_set(
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

        let ubo = UniformBufferObject {
            model: Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), time * 90.0_f32.to_radians()),
            view: Mat4::look_at_rh(vec3(2.0, 2.0, 2.0), vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0)),
            proj: Mat4::perspective_rh(
                45.0_f32.to_radians(),
                width as f32 / height as f32,
                0.1,
                10.0,
            ),
        };

        let ub = self.uniform_buffers[self.sys.frame];
        self.sys
            .rhi
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
            self.sys.rhi.reset_command_buffer(cb, false)?;
            self.sys.rhi.cmd_begin(cb, CommandBufferBeginDesc::default())?;
            let rp = self.sys.swapchain_render_pass;
            let fb = self.sys.get_swapchain_framebuffer();
            let cv = ClearColor::new([0.0, 0.0, 0.0, 1.0]);
            self.sys.rhi.cmd_begin_render_pass(cb, rp, fb, Some(&[cv]), None)?;
            self.sys.rhi.cmd_bind_raster_pipeline(cb, self.pipeline)?;
            self.sys.rhi.cmd_set_viewport(cb, 0.0, 0.0, width as f32, height as f32, 0.0, 1.0)?;
            self.sys.rhi.cmd_set_scissor(cb, 0, 0, width, height)?;
            self.sys.rhi.cmd_bind_vertex_buffers(cb, 0, &[self.vertex_buffer], &[0])?;
            self.sys.rhi.cmd_bind_index_buffer(cb, self.index_buffer, 0, IndexType::U16)?;
            self.sys.rhi.cmd_bind_descriptor_sets(
                cb,
                PipelineBindPoint::Graphics,
                self.pipeline_layout,
                0,
                &[self.descriptor_sets[self.sys.frame]],
                &[],
            )?;
            self.sys.rhi.cmd_draw_indexed(cb, self.indices.len() as u32, 1, 0, 0, 0)?;
            self.sys.rhi.cmd_end_render_pass(cb)?;
            self.sys.rhi.cmd_end(cb)?;
            self.sys.rhi.queue_submit(
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
        self.sys.rhi.device_wait_idle().unwrap();

        self.sys.rhi.destroy_image(self.image).unwrap();
        self.sys.rhi.destroy_sampler(self.sampler).unwrap();
        self.sys.rhi.destroy_buffer(self.vertex_buffer).unwrap();
        self.sys.rhi.destroy_buffer(self.index_buffer).unwrap();
        for ub in self.uniform_buffers.iter() {
            self.sys.rhi.destroy_buffer(*ub).unwrap();
        }

        self.sys.rhi.destroy_shader_module(self.vs).unwrap();
        self.sys.rhi.destroy_shader_module(self.fs).unwrap();
        self.sys.rhi.destroy_descriptor_set_layout(self.descriptor_set_layout).unwrap();

        self.sys.rhi.destroy_descriptor_pool(self.descriptor_pool).unwrap();
        self.sys.rhi.destroy_pipeline_layout(self.pipeline_layout).unwrap();
        self.sys.rhi.destroy_raster_pipeline(self.pipeline).unwrap();

        self.sys.destroy().unwrap();
    }
}
