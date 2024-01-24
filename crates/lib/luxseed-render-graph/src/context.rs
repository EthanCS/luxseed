use luxseed_render_backend::{define::CommandBuffer, RenderBackend};
use luxseed_utility::pool::Handle;

pub struct RenderGraphContext<'a> {
    pub backend: &'a mut dyn RenderBackend,
    pub command_buffer: Handle<CommandBuffer>,
}
