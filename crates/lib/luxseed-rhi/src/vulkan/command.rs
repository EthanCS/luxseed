use anyhow::{Context, Ok};
use ash::vk::{self};
use smallvec::SmallVec;

use crate::{
    define::*,
    enums::*,
    flag::PipelineStageFlags,
    impl_handle,
    pool::{Handle, Handled, Pool},
    MAX_RENDER_TARGETS,
};

use super::{
    buffer::VulkanBuffer,
    device::{VulkanDevice, VulkanQueue},
    framebuffer::VulkanFramebuffer,
    image::VulkanImage,
    pipeline::{VulkanPipelineLayout, VulkanRasterPipeline},
    render_pass::VulkanRenderPass,
};

#[derive(Default)]
pub struct VulkanCommandPool {
    pub handle: Option<Handle<CommandPool>>,
    pub raw: vk::CommandPool,
    pub device: Option<Handle<Device>>,
}
impl_handle!(VulkanCommandPool, CommandPool, handle);

impl VulkanCommandPool {
    pub fn init(&mut self, queue: &VulkanQueue, d: &VulkanDevice) -> anyhow::Result<()> {
        let raw = unsafe {
            d.raw().create_command_pool(
                &vk::CommandPoolCreateInfo::builder()
                    .queue_family_index(queue.family_index)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                    .build(),
                None,
            )?
        };
        self.device = d.get_handle();
        self.raw = raw;
        Ok(())
    }

    pub fn destroy(&mut self, d: &VulkanDevice) {
        unsafe {
            d.raw().destroy_command_pool(self.raw, None);
        }
        self.device = None;
        self.raw = vk::CommandPool::null();
    }

    pub fn reset(&self, device: &VulkanDevice) -> anyhow::Result<()> {
        unsafe {
            device.raw().reset_command_pool(self.raw, vk::CommandPoolResetFlags::empty())?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct VulkanCommandBuffer {
    pub handle: Option<Handle<CommandBuffer>>,
    pub raw: vk::CommandBuffer,
    pub device: Option<Handle<Device>>,
    pub pool: Option<Handle<CommandPool>>,
    cache_render_pass: Option<Handle<RenderPass>>,
    cache_framebuffer: Option<Handle<Framebuffer>>,
}
impl_handle!(VulkanCommandBuffer, CommandBuffer, handle);

impl VulkanCommandBuffer {
    pub fn init(
        &mut self,
        device: &VulkanDevice,
        pool: &VulkanCommandPool,
        level: CommandBufferLevel,
    ) -> anyhow::Result<()> {
        let raw = unsafe {
            device.raw().allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::builder()
                    .command_pool(pool.raw)
                    .level(level.into())
                    .command_buffer_count(1)
                    .build(),
            )?
        }[0];
        self.raw = raw;
        self.device = device.get_handle();
        self.pool = pool.get_handle();
        self.cache_framebuffer = None;
        self.cache_render_pass = None;
        Ok(())
    }

    #[inline]
    pub fn begin(&self, device: &VulkanDevice, desc: CommandBufferBeginDesc) -> anyhow::Result<()> {
        let mut flag = vk::CommandBufferUsageFlags::empty();
        if desc.one_time_submit {
            flag |= vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT;
        }
        let begin_info = vk::CommandBufferBeginInfo::builder().flags(flag).build();
        unsafe {
            device.raw().begin_command_buffer(self.raw, &begin_info)?;
        }
        Ok(())
    }

    #[inline]
    pub fn end(&self, device: &VulkanDevice) -> anyhow::Result<()> {
        unsafe {
            device.raw().end_command_buffer(self.raw)?;
        }
        Ok(())
    }

    #[inline]
    pub fn begin_render_pass(
        &self,
        device: &VulkanDevice,
        render_pass: &VulkanRenderPass,
        framebuffer: &VulkanFramebuffer,
        clear_values: Option<&[ClearColor]>,
        clear_depth_stencil: Option<ClearDepthStencil>,
    ) -> anyhow::Result<()> {
        let mut vk_clear_values = [vk::ClearValue::default(); MAX_RENDER_TARGETS + 1];
        {
            if clear_values.is_some() {
                let values = clear_values.unwrap();
                for attachment in 0..render_pass.output.num_colors {
                    vk_clear_values[attachment as usize] = values[attachment as usize].into();
                }
            }

            if clear_depth_stencil.is_some() {
                let value = clear_depth_stencil.unwrap();
                vk_clear_values[render_pass.output.num_colors as usize] = value.into();
            }
        }

        let create_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass.raw)
            .framebuffer(framebuffer.raw)
            .render_area(
                vk::Rect2D::builder()
                    .offset(vk::Offset2D::builder().x(0).y(0).build())
                    .extent(
                        vk::Extent2D::builder()
                            .width(framebuffer.desc.width.into())
                            .height(framebuffer.desc.height.into())
                            .build(),
                    )
                    .build(),
            )
            .clear_values(&vk_clear_values)
            .build();

        unsafe {
            device.raw().cmd_begin_render_pass(self.raw, &create_info, vk::SubpassContents::INLINE);
        }

        Ok(())
    }

    #[inline]
    pub fn end_render_pass(&self, device: &VulkanDevice) -> anyhow::Result<()> {
        unsafe {
            device.raw().cmd_end_render_pass(self.raw);
        }
        Ok(())
    }

    #[inline]
    pub fn bind_raster_pipeline(
        &self,
        device: &VulkanDevice,
        pipeline: &VulkanRasterPipeline,
    ) -> anyhow::Result<()> {
        unsafe {
            device.raw().cmd_bind_pipeline(self.raw, vk::PipelineBindPoint::GRAPHICS, pipeline.raw);
        }
        Ok(())
    }

    #[inline]
    pub fn set_viewport(
        &self,
        device: &VulkanDevice,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) -> anyhow::Result<()> {
        let viewport = vk::Viewport::builder()
            .x(x)
            .y(y)
            .width(width)
            .height(height)
            .min_depth(min_depth)
            .max_depth(max_depth)
            .build();
        unsafe {
            device.raw().cmd_set_viewport(self.raw, 0, &[viewport]);
        }
        Ok(())
    }

    #[inline]
    pub fn set_scissor(
        &self,
        device: &VulkanDevice,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D::builder().x(x as i32).y(y as i32).build())
            .extent(vk::Extent2D::builder().width(width).height(height).build())
            .build();
        unsafe {
            device.raw().cmd_set_scissor(self.raw, 0, &[scissor]);
        }
        Ok(())
    }

    #[inline]
    pub fn draw(
        &self,
        device: &VulkanDevice,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            device.raw().cmd_draw(
                self.raw,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        }
    }

    #[inline]
    pub fn draw_indexed(
        &self,
        device: &VulkanDevice,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            device.raw().cmd_draw_indexed(
                self.raw,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }

    #[inline]
    pub fn bind_descriptor_sets(
        &self,
        device: &VulkanDevice,
        pipeline_layout: &VulkanPipelineLayout,
        first_set: u32,
        descriptor_sets: &[vk::DescriptorSet],
        dynamic_offsets: &[u32],
    ) {
        unsafe {
            device.raw().cmd_bind_descriptor_sets(
                self.raw,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout.raw,
                first_set,
                descriptor_sets,
                dynamic_offsets,
            );
        }
    }

    #[inline]
    pub fn copy_buffer(
        &self,
        device: &VulkanDevice,
        src: &VulkanBuffer,
        dst: &VulkanBuffer,
        regions: &[BufferCopyRegion],
    ) {
        let mut v = SmallVec::<[vk::BufferCopy; 4]>::new();
        for region in regions {
            v.push((*region).into());
        }
        unsafe {
            device.raw().cmd_copy_buffer(self.raw, src.raw, dst.raw, &v);
        }
    }

    #[inline]
    pub fn copy_buffer_to_image(
        &self,
        device: &VulkanDevice,
        src: &VulkanBuffer,
        dst: &VulkanImage,
        dst_image_layout: ImageLayout,
        regions: &[BufferImageCopyRegion],
    ) {
        let mut regions_ = SmallVec::<[vk::BufferImageCopy; 4]>::new();
        for region in regions {
            regions_.push(
                vk::BufferImageCopy::builder()
                    .buffer_offset(region.buffer_offset)
                    .buffer_image_height(region.buffer_image_height)
                    .buffer_row_length(region.buffer_row_length)
                    .image_subresource(
                        vk::ImageSubresourceLayers::builder()
                            .aspect_mask(region.aspect_mask.into())
                            .mip_level(region.mip_level)
                            .base_array_layer(region.base_array_layer)
                            .layer_count(region.layer_count)
                            .build(),
                    )
                    .image_offset(vk::Offset3D {
                        x: region.image_offset[0],
                        y: region.image_offset[1],
                        z: region.image_offset[2],
                    })
                    .image_extent(vk::Extent3D {
                        width: region.image_extent[0],
                        height: region.image_extent[1],
                        depth: region.image_extent[2],
                    })
                    .build(),
            );
        }
        unsafe {
            device.raw().cmd_copy_buffer_to_image(
                self.raw,
                src.raw,
                dst.raw,
                dst_image_layout.into(),
                &regions_,
            );
        }
    }

    #[inline]
    pub fn pipeline_barrier(
        &self,
        device: &VulkanDevice,
        src_stage: PipelineStageFlags,
        dst_stage: PipelineStageFlags,
        image_barriers: &[ImageMemoryBarrier],
        p_image: &Pool<VulkanImage>,
    ) -> anyhow::Result<()> {
        let mut image_memory_barriers = SmallVec::<[vk::ImageMemoryBarrier; 4]>::new();
        for barrier in image_barriers {
            image_memory_barriers.push(
                vk::ImageMemoryBarrier::builder()
                    .old_layout(barrier.old_layout.into())
                    .new_layout(barrier.new_layout.into())
                    .src_queue_family_index(
                        barrier.src_queue_family_index.unwrap_or(vk::QUEUE_FAMILY_IGNORED),
                    )
                    .dst_queue_family_index(
                        barrier.dst_queue_family_index.unwrap_or(vk::QUEUE_FAMILY_IGNORED),
                    )
                    .image(p_image.get(barrier.image).context("Image not found")?.raw)
                    .subresource_range(
                        vk::ImageSubresourceRange::builder()
                            .aspect_mask(barrier.aspect_mask.into())
                            .base_mip_level(barrier.base_mip_level)
                            .level_count(barrier.level_count)
                            .base_array_layer(barrier.base_array_layer)
                            .layer_count(barrier.layer_count)
                            .build(),
                    )
                    .src_access_mask(vk::AccessFlags::empty())
                    .dst_access_mask(vk::AccessFlags::empty())
                    .build(),
            );
        }
        unsafe {
            device.raw().cmd_pipeline_barrier(
                self.raw,
                src_stage.into(),
                dst_stage.into(),
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &image_memory_barriers,
            );
        }
        Ok(())
    }

    #[inline]
    pub fn reset(&self, device: &VulkanDevice, release: bool) -> anyhow::Result<()> {
        let flags = if release {
            vk::CommandBufferResetFlags::RELEASE_RESOURCES
        } else {
            vk::CommandBufferResetFlags::empty()
        };
        unsafe {
            device.raw().reset_command_buffer(self.raw, flags)?;
        }
        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice, pool: &VulkanCommandPool) {
        unsafe {
            device.raw().free_command_buffers(pool.raw, &[self.raw]);
        }
        self.raw = vk::CommandBuffer::null();
        self.device = None;
        self.pool = None;
        self.cache_framebuffer = None;
        self.cache_render_pass = None;
    }
}
