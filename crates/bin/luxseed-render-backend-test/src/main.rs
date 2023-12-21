mod render_system;

mod helloworld;

use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use helloworld::App;

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
