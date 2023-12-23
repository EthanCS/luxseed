use anyhow::Ok;
use ash::vk;

use crate::{define::RenderPass, impl_handle, pool::Handle, MAX_RENDER_TARGETS};

use super::device::VulkanDevice;

#[derive(Default)]
pub struct VulkanRenderPass {
    pub handle: Option<Handle<RenderPass>>,
    pub raw: vk::RenderPass,
    pub output: VulkanRenderPassOutput,
}
impl_handle!(VulkanRenderPass, RenderPass, handle);

impl VulkanRenderPass {
    pub fn init(&mut self, raw: vk::RenderPass, output: VulkanRenderPassOutput) {
        self.raw = raw;
        self.output = output;
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_render_pass(self.raw, None);
        }
        self.raw = vk::RenderPass::null();
        self.output = Default::default();
    }
}

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct VulkanRenderPassOutput {
    pub num_colors: u8,
    pub color_formats: [vk::Format; MAX_RENDER_TARGETS],
    pub color_final_layouts: [vk::ImageLayout; MAX_RENDER_TARGETS],
    pub color_load: [vk::AttachmentLoadOp; MAX_RENDER_TARGETS],
    pub color_samples: [vk::SampleCountFlags; MAX_RENDER_TARGETS],
    pub depth_stencil_format: vk::Format,
    pub depth_stencil_final_layout: vk::ImageLayout,
    pub depth_stencil_samples: vk::SampleCountFlags,
    pub depth_load: vk::AttachmentLoadOp,
    pub stencil_load: vk::AttachmentLoadOp,
}

impl VulkanDevice {
    pub fn get_or_create_render_pass(
        &mut self,
        layout: &VulkanRenderPassOutput,
    ) -> anyhow::Result<vk::RenderPass> {
        if let Some(rp) = self.render_pass_cache.get(&layout) {
            return Ok(*rp);
        }
        let new_rp = VulkanDevice::create_vulkan_render_pass(self.raw(), layout)?;
        self.render_pass_cache.insert(*layout, new_rp);
        return Ok(new_rp);
    }

    fn create_vulkan_render_pass(
        device: &ash::Device,
        layout: &VulkanRenderPassOutput,
    ) -> anyhow::Result<vk::RenderPass> {
        // Depth intial image layout
        let depth_initial = if layout.depth_load == vk::AttachmentLoadOp::DONT_CARE {
            vk::ImageLayout::UNDEFINED
        } else {
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        };

        // Color
        let mut color_attachments = Vec::new();
        let mut color_attachments_ref = Vec::new();
        for i in 0..layout.num_colors {
            let color_initial = if layout.color_load[i as usize] == vk::AttachmentLoadOp::LOAD {
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            } else {
                vk::ImageLayout::UNDEFINED
            };

            let color_attachment = vk::AttachmentDescription::builder()
                .format(layout.color_formats[i as usize])
                .samples(layout.color_samples[i as usize])
                .load_op(layout.color_load[i as usize])
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(layout.stencil_load)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(color_initial)
                .final_layout(layout.color_final_layouts[i as usize])
                .build();
            color_attachments.push(color_attachment);

            let color_attachment_ref = vk::AttachmentReference::builder()
                .attachment(i as u32)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build();
            color_attachments_ref.push(color_attachment_ref);
        }

        // Depth stencil
        let has_depth_stencil = layout.depth_stencil_format != vk::Format::UNDEFINED;
        let mut depth_attachment = Default::default();
        let mut depth_attachment_ref = Default::default();
        if has_depth_stencil {
            depth_attachment = vk::AttachmentDescription::builder()
                .format(layout.depth_stencil_format)
                .samples(layout.depth_stencil_samples)
                .load_op(layout.depth_load)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(layout.stencil_load)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(depth_initial)
                .final_layout(layout.depth_stencil_final_layout)
                .build();
            depth_attachment_ref = vk::AttachmentReference::builder()
                .attachment(layout.num_colors as u32)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build();
        }

        // Build attachment descriptions
        let mut attachment_descriptions = color_attachments;
        if has_depth_stencil {
            attachment_descriptions.push(depth_attachment);
        }

        // Build main subpass
        let mut subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments_ref);
        if has_depth_stencil {
            subpass = subpass.depth_stencil_attachment(&depth_attachment_ref);
        }
        let subpass = subpass.build();

        // // Build dependency
        // let dependency = vk::SubpassDependency::builder()
        //     .src_subpass(vk::SUBPASS_EXTERNAL)
        //     .dst_subpass(0)
        //     .src_stage_mask(
        //         vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
        //             | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        //     )
        //     .dst_stage_mask(
        //         vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
        //             | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        //     )
        //     .src_access_mask(vk::AccessFlags::empty())
        //     .dst_access_mask(
        //         vk::AccessFlags::COLOR_ATTACHMENT_WRITE
        //             | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        //     )
        //     .build();

        // Create Render Pass
        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .subpasses(&[subpass])
            .attachments(&attachment_descriptions)
            //.dependencies(&[dependency])
            .build();
        let render_pass = unsafe { device.create_render_pass(&render_pass_info, None)? };

        Ok(render_pass)
    }
}
