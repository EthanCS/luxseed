use ash::vk;

use crate::{
    define::{Device, RasterPipeline, RasterPipelineCreation},
    impl_handle,
    pool::{Handle, Handled, Pool},
};

use super::{device::VulkanDevice, shader::VulkanShader};

#[derive(Default)]
pub struct VulkanRasterPipeline {
    pub raw: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub handle: Option<Handle<RasterPipeline>>,
    pub device: Option<Handle<Device>>,
}
impl_handle!(VulkanRasterPipeline, RasterPipeline, handle);

impl VulkanRasterPipeline {
    pub fn init(
        &mut self,
        device: &VulkanDevice,
        render_pass: vk::RenderPass,
        desc: &RasterPipelineCreation,
        p_shader: &Pool<VulkanShader>,
    ) -> anyhow::Result<()> {
        // Vertex Input
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder().build();

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
        if desc.num_blend_states > 0 {
            for i in 0..desc.num_blend_states {
                let blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
                    .blend_enable(desc.blend_states[i as usize].blend_enable)
                    .src_color_blend_factor(desc.blend_states[i as usize].source_color.into())
                    .dst_color_blend_factor(desc.blend_states[i as usize].destination_color.into())
                    .color_blend_op(desc.blend_states[i as usize].color_op.into())
                    .src_alpha_blend_factor(desc.blend_states[i as usize].source_alpha.into())
                    .dst_alpha_blend_factor(desc.blend_states[i as usize].destination_alpha.into())
                    .alpha_blend_op(desc.blend_states[i as usize].alpha_op.into())
                    .build();
                color_blend_attachments.push(blend_attachment);
            }
        } else {
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
        for i in 0..desc.num_shader_stages {
            let shader = p_shader.get(desc.shader_stages[i as usize]).unwrap();
            let shader_stage = vk::PipelineShaderStageCreateInfo::builder()
                .stage(shader.stage.into())
                .module(shader.raw)
                .name(shader.entry.as_c_str())
                .build();
            shader_stages.push(shader_stage);
        }

        // Pipeline Layout
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            // .set_layouts(set_layouts)
            // .push_constant_ranges(push_constant_ranges)
            .build();
        let pipeline_layout =
            unsafe { device.raw().create_pipeline_layout(&pipeline_layout_info, None)? };
        self.pipeline_layout = pipeline_layout;

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
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1)
            .build();

        self.device = device.get_handle();
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
            device.raw().destroy_pipeline_layout(self.pipeline_layout, None);
            device.raw().destroy_pipeline(self.raw, None);
        }
        self.device = None;
        self.raw = vk::Pipeline::null();
        self.pipeline_layout = vk::PipelineLayout::null();
    }
}
