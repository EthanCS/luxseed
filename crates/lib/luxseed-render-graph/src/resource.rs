use luxseed_render_backend::{
    enums::{Format, RenderTargetLoadAction},
    flag::{BufferUsageFlags, ImageUsageFlags},
};
use luxseed_utility::pool::Handle;

use crate::node::Node;

pub struct Resource {
    pub producer: Handle<Node>,
    pub info: ResourceInfo,
    pub output: Handle<Resource>,
    pub name: String,
    pub ref_count: i32,
}

pub enum ResourceInfo {
    Buffer(BufferInfo),
    Image(ImageInfo),
}

pub struct BufferInfo {
    pub external: bool,
    pub handle: Handle<BufferInfo>,
    pub size: usize,
    pub usage: BufferUsageFlags,
}

pub struct ImageInfo {
    pub external: bool,
    pub handle: Handle<ImageInfo>,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub format: Format,
    pub usage: ImageUsageFlags,
    pub load_op: RenderTargetLoadAction,
}
