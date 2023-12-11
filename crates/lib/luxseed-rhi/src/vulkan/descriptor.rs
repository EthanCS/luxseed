use anyhow::{Context, Result};
use ash::vk;
use smallvec::SmallVec;

use crate::{
    define::*,
    enums::DescriptorType,
    impl_handle,
    pool::{Handle, Handled, Pool},
};

use super::{buffer::VulkanBuffer, device::VulkanDevice};

#[derive(Default)]
pub struct VulkanDescriptorSetLayout {
    pub handle: Option<Handle<DescriptorSetLayout>>,
    pub raw: vk::DescriptorSetLayout,
    pub device: Option<Handle<Device>>,
}
impl_handle!(VulkanDescriptorSetLayout, DescriptorSetLayout, handle);

impl VulkanDescriptorSetLayout {
    pub fn init(
        &mut self,
        device: &VulkanDevice,
        desc: &DescriptorSetLayoutCreateDesc,
    ) -> Result<()> {
        let mut bindings = SmallVec::<[vk::DescriptorSetLayoutBinding; 8]>::new();
        for b in desc.bindings.iter() {
            let binding = vk::DescriptorSetLayoutBinding::builder()
                .binding(b.binding)
                .descriptor_type(b.descriptor_type.into())
                .descriptor_count(b.descriptor_count)
                .stage_flags(b.stage_flags.into());
            bindings.push(binding.build());
        }
        let raw = unsafe {
            device.raw().create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings).build(),
                None,
            )?
        };
        self.raw = raw;
        self.device = device.get_handle();
        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_descriptor_set_layout(self.raw, None);
        }
        self.raw = vk::DescriptorSetLayout::null();
        self.device = None;
    }
}

#[derive(Default)]
pub struct VulkanDescriptorPool {
    pub handle: Option<Handle<DescriptorPool>>,
    pub raw: vk::DescriptorPool,
    pub device: Option<Handle<Device>>,
}
impl_handle!(VulkanDescriptorPool, DescriptorPool, handle);

impl VulkanDescriptorPool {
    pub fn init(&mut self, device: &VulkanDevice, desc: &DescriptorPoolCreateDesc) -> Result<()> {
        let mut pool_sizes = SmallVec::<[vk::DescriptorPoolSize; 8]>::new();
        for p in desc.pool_sizes.iter() {
            let pool_size = vk::DescriptorPoolSize::builder()
                .ty(p.descriptor_type.into())
                .descriptor_count(p.descriptor_count);
            pool_sizes.push(pool_size.build());
        }
        let raw = unsafe {
            device.raw().create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::builder()
                    .pool_sizes(&pool_sizes)
                    .max_sets(desc.max_sets)
                    .build(),
                None,
            )?
        };
        self.raw = raw;
        self.device = device.get_handle();
        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_descriptor_pool(self.raw, None);
        }
        self.raw = vk::DescriptorPool::null();
        self.device = None;
    }
}

#[derive(Default)]
pub struct VulkanDescriptorSet {
    pub handle: Option<Handle<DescriptorSet>>,
    pub raw: vk::DescriptorSet,
    pub device: Option<Handle<Device>>,
    pub pool: Option<Handle<DescriptorPool>>,
}
impl_handle!(VulkanDescriptorSet, DescriptorSet, handle);

impl VulkanDescriptorSet {
    pub fn init(
        &mut self,
        device: &VulkanDevice,
        pool: &VulkanDescriptorPool,
        layout: &VulkanDescriptorSetLayout,
    ) -> Result<()> {
        let raw = unsafe {
            device.raw().allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::builder()
                    .descriptor_pool(pool.raw)
                    .set_layouts(&[layout.raw])
                    .build(),
            )?
        }[0];
        self.raw = raw;
        self.pool = pool.get_handle();
        self.device = device.get_handle();
        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice, pool: &VulkanDescriptorPool) -> Result<()> {
        unsafe {
            device.raw().free_descriptor_sets(pool.raw, &[self.raw])?;
        }
        self.raw = vk::DescriptorSet::null();
        self.pool = None;
        self.device = None;
        Ok(())
    }
}

impl VulkanDevice {
    pub fn update_descriptor_sets(
        &self,
        writes: &[DescriptorSetWriteDesc],
        _copies: &[DescriptorSetCopyDesc],
        p_buffer: &Pool<VulkanBuffer>,
        p_descriptor_sets: &Pool<VulkanDescriptorSet>,
    ) -> Result<()> {
        // For writes
        let mut dsw = SmallVec::<[vk::WriteDescriptorSet; 8]>::new();
        let mut write_buffer_infos = SmallVec::<[vk::DescriptorBufferInfo; 8]>::new();
        let mut write_image_infos = SmallVec::<[vk::DescriptorImageInfo; 8]>::new();

        for w in writes.iter() {
            let mut dst_set = vk::WriteDescriptorSet::builder()
                .dst_set(p_descriptor_sets.get(w.dst_set).context("Descriptor Set not found")?.raw)
                .dst_binding(w.dst_binding)
                .dst_array_element(w.dst_array_element)
                .descriptor_type(w.descriptor_type.into());

            match w.descriptor_type {
                DescriptorType::UniformBuffer | DescriptorType::UniformBufferDynamic => {
                    for b in w.buffer_infos.iter() {
                        let buffer_info = vk::DescriptorBufferInfo::builder()
                            .buffer(p_buffer.get(b.buffer).context("Buffer not found")?.raw)
                            .offset(b.offset)
                            .range(b.range)
                            .build();
                        write_buffer_infos.push(buffer_info);
                    }
                    dst_set = dst_set.buffer_info(&write_buffer_infos);
                }
                // DescriptorType::CombinedImageSampler => {
                //     for i in w.image_infos.iter() {
                //         let image_info = vk::DescriptorImageInfo::builder()
                //             .image_layout(i.image_layout)
                //             .image_view(i.image_view)
                //             .sampler(i.sampler)
                //             .build();
                //         write_image_infos.push(image_info);
                //     }
                //     dst_set = dst_set.image_info(&write_image_infos);
                // }
                _ => {
                    return Err(anyhow::anyhow!("Unsupported descriptor type"));
                }
            }

            dsw.push(dst_set.build());
        }

        // For copies
        let dsc = SmallVec::<[vk::CopyDescriptorSet; 8]>::new();

        unsafe {
            self.raw().update_descriptor_sets(&dsw, &dsc);
        }
        Ok(())
    }
}
