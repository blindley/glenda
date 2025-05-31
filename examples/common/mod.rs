
// the gl crate is exported publicly
use glume::gl;

use glenda::renderers::{
    Renderer,
    Viewport,
};

pub fn run_example<A: Application + 'static>() -> Result<(), Box<dyn std::error::Error>> {
    // initial configuration for the window
    let window_config = A::window_configuration();

    let window = window_config.build_window();

    // after the window is created, we can call OpenGL functions, not before
    unsafe {
        gl::DebugMessageCallback(
            Some(glh::standard_debug_callback),
            std::ptr::null_mut(),
        );
        gl::Enable(gl::DEBUG_OUTPUT);
    }

    let mut app = A::new()?;

    window.run(move |wc, event| {
        use glume::window::Event;
        match event {
            Event::Resized(width, height) => {
                let viewport = Viewport::from([width as i32, height as i32]);
                app.set_viewport(viewport);
            }

            Event::RedrawRequested => {
                app.render();
            }

            Event::KeyPressed(key) => {
                use glume::window::VirtualKeyCode as Vk;
                match key {
                    Vk::Escape => wc.close(),
                    k => app.key_pressed(k),
                }
            }

            _ => (),
        }

        Ok(())
    });
}

pub trait Application : Renderer {
    fn window_configuration() -> glume::window::WindowConfiguration;
    fn new() -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;

    #[allow(unused_variables)]
    fn key_pressed(&mut self, key: glume::window::VirtualKeyCode) {}
}
