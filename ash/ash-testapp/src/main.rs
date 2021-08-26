mod vulkan_app;
mod constants;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = init_window(&event_loop);
    let vulkan_app = vulkan_app::VulkanApp::new(&window);

    main_loop(event_loop, window, vulkan_app);
}

fn init_window(event_loop: &winit::event_loop::EventLoop<()>) -> winit::window::Window {
    winit::window::WindowBuilder::new()
        .with_title(constants::WINDOW_TITLE)
        .with_inner_size(winit::dpi::LogicalSize::new(
            constants::WINDOW_WIDTH,
            constants::WINDOW_HEIGHT,
        ))
        .build(event_loop)
        .expect("Failed to create window.")
}

fn main_loop(
    event_loop: winit::event_loop::EventLoop<()>,
    window: winit::window::Window,
    mut vulkan_app: vulkan_app::VulkanApp,
) {
    use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
    use winit::event_loop::ControlFlow;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput { virtual_keycode, state, .. } => match (virtual_keycode, state) {
                    (Some(VirtualKeyCode::Escape), ElementState::Released) => *control_flow = ControlFlow::Exit,
                    _ => {}
                },
            },
            _ => {}
        },
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        Event::RedrawRequested(_window_id) => {
            vulkan_app.draw();
        }
        _ => {}
    })
}