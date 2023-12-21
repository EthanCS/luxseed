use anyhow::{Context, Result};
use ash::vk;

use crate::{
    define::{PipelineLayout, PipelineLayoutCreateDesc, RasterPipeline, RasterPipelineCreateDesc},
    impl_handle,
    pool::{Handle, Pool},
};

use super::{descriptor::VulkanDescriptorSetLayout, device::VulkanDevice, shader::VulkanShader};

#[derive(Default)]
pub struct VulkanPipelineLayout {
    pub handle: Option<Handle<PipelineLayout>>,
    pub raw: vk::PipelineLayout,
}
impl_handle!(VulkanPipelineLayout, PipelineLayout, handle);

impl VulkanPipelineLayout {
    pub fn init(
        &mut self,
        device: &VulkanDevice,
        desc: &PipelineLayoutCreateDesc,
        p_descriptor_set_layout: &Pool<VulkanDescriptorSetLayout>,
    ) -> Result<()> {
        let mut set_layouts = smallvec::SmallVec::<[vk::DescriptorSetLayout; 4]>::new();
        for handle in desc.descriptor_set_layouts {
            let layout =
                p_descriptor_set_layout.get(*handle).context("Descriptor set layout not found")?;
            set_layouts.push(layout.raw);
        }

        let raw = unsafe {
            device.raw().create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::builder()
                    .set_layouts(&set_layouts)
                    .push_constant_ranges(&[])
                    .build(),
                None,
            )?
        };
        self.raw = raw;
        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_pipeline_layout(self.raw, None);
        }
        self.raw = vk::PipelineLayout::null();
    }
}

#[derive(Default)]
pub struct VulkanRasterPipeline {
    pub raw: vk::Pipeline,
    pub handle: Option<Handle<RasterPipeline>>,
}
impl_handle!(VulkanRasterPipeline, RasterPipeline, handle);

impl VulkanRasterPipeline {
    pub fn init(
        &mut self,
        device: &VulkanDevice,
        render_pass: vk::RenderPass,
        pipeline_layout: &VulkanPipelineLayout,
        desc: &RasterPipelineCreateDesc,
        p_shader: &Pool<VulkanShader>,
    ) -> anyhow::Result<()> {
        // Vertex Input
        let mut vertex_input_bindings = Vec::new();
        let mut vertex_input_attributes = Vec::new();

        if let Some(bindings) = desc.vertex_input_bindings {
            let mut binding: u32 = 0;

            for b in bindings.iter() {
                let vib = vk::VertexInputBindingDescription::builder()
                    .binding(binding)
                    .stride(b.stride as u32)
                    .input_rate(b.input_rate.into())
                    .build();

                let mut location: u32 = 0;
                for a in b.attributes.iter() {
                    let via = vk::VertexInputAttributeDescription::builder()
                        .binding(binding)
                        .location(location)
                        .format(a.format.into())
                        .offset(a.offset as u32)
                        .build();
                    vertex_input_attributes.push(via);
                    location = location + 1;
                }

                vertex_input_bindings.push(vib);
                binding = binding + 1;
            }
        }

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&vertex_input_bindings)
            .vertex_attribute_descriptions(&vertex_input_attributes)
            .build();

        // Input Assembly State
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false)
            .build();

        // Viewport and Scissor
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR])
            .build();
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1)
            .build();

        // Rasterization State
        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .cull_mode(desc.raster_state.cull_mode.into())
            .front_face(desc.raster_state.front_face.into())
            .polygon_mode(desc.raster_state.fill_mode.into())
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .line_width(1.0)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0)
            .build();

        // Multisample State
        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .min_sample_shading(1.0)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false)
            .build();

        // Depth Stencil State
        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(desc.depth_state.depth_test_enable)
            .depth_write_enable(desc.depth_state.depth_write_enable)
            .depth_compare_op(desc.depth_state.depth_compare_mode.into())
            .stencil_test_enable(desc.depth_state.stencil_test_enable)
            .front(desc.depth_state.stencil_front.into())
            .back(desc.depth_state.stencil_back.into())
            .build();

        // Color blend State
        let mut color_blend_attachments = Vec::new();
        let mut num_blend_states = 0;

        for bs in desc.blend_states.iter() {
            let blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .blend_enable(bs.blend_enable)
                .src_color_blend_factor(bs.source_color.into())
                .dst_color_blend_factor(bs.destination_color.into())
                .color_blend_op(bs.color_op.into())
                .src_alpha_blend_factor(bs.source_alpha.into())
                .dst_alpha_blend_factor(bs.destination_alpha.into())
                .alpha_blend_op(bs.alpha_op.into())
                .build();
            color_blend_attachments.push(blend_attachment);
            num_blend_states = num_blend_states + 1;
        }

        if num_blend_states == 0 {
            color_blend_attachments.push(
                vk::PipelineColorBlendAttachmentState::builder()
                    .blend_enable(false)
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
                    .build(),
            );
        }

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build();

        // Shader stages
        let mut shader_stages = Vec::new();
        for handle in desc.shader_stages.iter() {
            let shader = p_shader.get(*handle).context("Shader not found")?;
            let shader_stage = vk::PipelineShaderStageCreateInfo::builder()
                .stage(shader.stage.into())
                .module(shader.raw)
                .name(shader.entry.as_c_str())
                .build();
            shader_stages.push(shader_stage);
        }

        // Finish setting up the pipeline and create it
        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .dynamic_state(&dynamic_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .multisample_state(&multisample_state)
            .stages(&shader_stages)
            .layout(pipeline_layout.raw)
            .render_pass(render_pass)
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1)
            .build();

        self.raw = unsafe {
            device
                .raw()
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|e| anyhow::anyhow!("Failed to create graphics pipeline: {:?}", e.1))?[0]
        };
        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_pipeline(self.raw, None);
        }
        self.raw = vk::Pipeline::null();
    }
}
