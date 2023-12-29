use anyhow::{Context, Result};
use ash::vk;
use luxseed_utility::impl_handle;
use luxseed_utility::pool::{Handle, Handled, Pool};
use smallvec::SmallVec;

use crate::{define::*, enums::DescriptorType};

use super::{
    buffer::VulkanBuffer,
    device::VulkanDevice,
    image::{VulkanImageView, VulkanSampler},
};

#[derive(Default)]
pub struct VulkanDescriptorSetLayout {
    pub handle: Option<Handle<DescriptorSetLayout>>,
    pub raw: vk::DescriptorSetLayout,
    pub vk_bindings: SmallVec<[vk::DescriptorSetLayoutBinding; MAX_DESCRIPTORS_PER_SET]>,
    pub binding_infos: SmallVec<[DescriptorBindingInfo; MAX_DESCRIPTORS_PER_SET]>,
    pub index_to_binding: SmallVec<[u8; MAX_DESCRIPTORS_PER_SET]>,
}
impl_handle!(VulkanDescriptorSetLayout, DescriptorSetLayout, handle);

impl VulkanDescriptorSetLayout {
    pub fn init(
        &mut self,
        device: &VulkanDevice,
        desc: &DescriptorSetLayoutCreateDesc,
    ) -> Result<()> {
        self.index_to_binding.clear();
        self.binding_infos.clear();
        self.vk_bindings.clear();

        self.index_to_binding.resize(desc.bindings.len(), 0);

        for (idx, input_binding) in desc.bindings.iter().enumerate() {
            let binding_index =
                if input_binding.index == u16::MAX { idx as u16 } else { input_binding.index };
            let binding = DescriptorBindingInfo {
                index: binding_index,
                type_: input_binding.type_,
                count: input_binding.count,
                stage_flags: input_binding.stage_flags,
            };
            self.binding_infos.push(binding);
            self.index_to_binding[binding.index as usize] = idx as u8;

            let vk_binding = vk::DescriptorSetLayoutBinding::builder()
                .binding(binding.index as u32)
                .descriptor_type(binding.type_.into())
                .descriptor_count(binding.count as u32)
                .stage_flags(binding.stage_flags.into())
                .build();
            self.vk_bindings.push(vk_binding);
        }

        let raw = unsafe {
            device.raw().create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::builder().bindings(&self.vk_bindings).build(),
                None,
            )?
        };
        self.raw = raw;
        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_descriptor_set_layout(self.raw, None);
        }
        self.raw = vk::DescriptorSetLayout::null();
        self.index_to_binding.clear();
        self.binding_infos.clear();
        self.vk_bindings.clear();
    }

    #[inline]
    pub fn get_binding_info(&self, binding: u16) -> Option<&DescriptorBindingInfo> {
        let binding_index = self.index_to_binding[binding as usize];
        self.binding_infos.get(binding_index as usize)
    }
}

#[derive(Default)]
pub struct VulkanDescriptorPool {
    pub handle: Option<Handle<DescriptorPool>>,
    pub raw: vk::DescriptorPool,
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
        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.raw().destroy_descriptor_pool(self.raw, None);
        }
        self.raw = vk::DescriptorPool::null();
    }
}

#[derive(Default)]
pub struct VulkanDescriptorSet {
    pub handle: Option<Handle<DescriptorSet>>,
    pub raw: vk::DescriptorSet,
    pub pool: Option<Handle<DescriptorPool>>,
    pub layout: Option<Handle<DescriptorSetLayout>>,
    pub binding_datas: SmallVec<[DescriptorBindingData; MAX_DESCRIPTORS_PER_SET]>,
}
impl_handle!(VulkanDescriptorSet, DescriptorSet, handle);

impl VulkanDescriptorSet {
    pub fn init(
        &mut self,
        device: &VulkanDevice,
        desc: &DescriptorSetCreateDesc,
        p_pool: &Pool<VulkanDescriptorPool>,
        p_layout: &Pool<VulkanDescriptorSetLayout>,
        p_buffer: &Pool<VulkanBuffer>,
        p_image_view: &Pool<VulkanImageView>,
        p_sampler: &Pool<VulkanSampler>,
    ) -> Result<()> {
        let pool = p_pool.get(desc.pool).context("Descriptor Pool not found")?;
        let layout = p_layout.get(desc.layout).context("Descriptor Set Layout not found")?;
        let raw = unsafe {
            device.raw().allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::builder()
                    .descriptor_pool(pool.raw)
                    .set_layouts(&[layout.raw])
                    .build(),
            )?
        }[0];

        // Update descriptor sets
        let mut writes = SmallVec::<[vk::WriteDescriptorSet; MAX_DESCRIPTORS_PER_SET]>::new();
        let mut buffers = SmallVec::<[vk::DescriptorBufferInfo; MAX_DESCRIPTORS_PER_SET]>::new();
        let mut images = SmallVec::<[vk::DescriptorImageInfo; MAX_DESCRIPTORS_PER_SET]>::new();
        device.fill_write_descriptor_sets(
            layout,
            raw,
            &desc.bindings,
            p_buffer,
            p_image_view,
            p_sampler,
            &mut writes,
            &mut buffers,
            &mut images,
        )?;
        unsafe {
            device.raw().update_descriptor_sets(&writes, &[]);
        }

        self.raw = raw;
        self.pool = pool.get_handle();
        self.layout = layout.get_handle();
        self.binding_datas.clear();
        for b in desc.bindings.iter() {
            self.binding_datas.push(*b);
        }

        Ok(())
    }

    pub fn destroy(
        &mut self,
        device: &ash::Device,
        p_pool: &Pool<VulkanDescriptorPool>,
    ) -> Result<()> {
        let pool = p_pool.get(self.pool.unwrap()).context("Descriptor Pool not found")?;
        unsafe {
            device.free_descriptor_sets(pool.raw, &[self.raw])?;
        }
        self.raw = vk::DescriptorSet::null();
        self.pool = None;
        self.layout = None;
        self.binding_datas.clear();
        Ok(())
    }
}

impl VulkanDevice {
    pub fn fill_write_descriptor_sets(
        &self,
        layout: &VulkanDescriptorSetLayout,
        descriptor_set: vk::DescriptorSet,
        binding_datas: &SmallVec<[DescriptorBindingData; MAX_DESCRIPTORS_PER_SET]>,
        p_buffer: &Pool<VulkanBuffer>,
        p_image_view: &Pool<VulkanImageView>,
        p_sampler: &Pool<VulkanSampler>,
        write_sets: &mut SmallVec<[vk::WriteDescriptorSet; MAX_DESCRIPTORS_PER_SET]>,
        buffer_infos: &mut SmallVec<[vk::DescriptorBufferInfo; MAX_DESCRIPTORS_PER_SET]>,
        image_infos: &mut SmallVec<[vk::DescriptorImageInfo; MAX_DESCRIPTORS_PER_SET]>,
    ) -> Result<()> {
        for binding_data in binding_datas.iter() {
            let binding_info =
                layout.get_binding_info(binding_data.binding).context("Can't find binding")?;

            let mut dst_set = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set)
                .dst_binding(binding_data.binding as u32)
                .dst_array_element(0)
                .descriptor_type(binding_info.type_.into());
            match binding_info.type_ {
                DescriptorType::UniformBuffer => {
                    let buffer_start_index = buffer_infos.len();

                    let buffer =
                        p_buffer.get(binding_data.buffer.unwrap()).context("Buffer not found")?;
                    let buffer_info = vk::DescriptorBufferInfo::builder()
                        .buffer(buffer.raw)
                        .offset(0)
                        .range(buffer.size)
                        .build();
                    buffer_infos.push(buffer_info);

                    dst_set = dst_set.buffer_info(&buffer_infos[buffer_start_index..]);
                }
                DescriptorType::CombinedImageSampler => {
                    let image_start_index = image_infos.len();

                    let image_view = p_image_view
                        .get(binding_data.image_view.unwrap())
                        .context("Image View not found")?;
                    let sampler = p_sampler
                        .get(binding_data.sampler.unwrap())
                        .context("Sampler not found")?;
                    let image_info = vk::DescriptorImageInfo::builder()
                        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .image_view(image_view.raw)
                        .sampler(sampler.raw)
                        .build();
                    image_infos.push(image_info);

                    dst_set = dst_set.image_info(&image_infos[image_start_index..]);
                }
                _ => {
                    todo!()
                }
            }

            write_sets.push(dst_set.build());
        }

        Ok(())
    }
}
