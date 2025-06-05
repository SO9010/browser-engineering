#![deny(clippy::all)]
#![forbid(unsafe_code)]

use pixels::{Error, Pixels, SurfaceTexture};
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use crate::layout::text::Body;

const WIDTH: u32 = 500;
const HEIGHT: u32 = 500;

// Should parse renderer arguments here. Like show all
pub fn init_renderer(body: Body) -> Result<(), Error> {
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Raqote")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let (mut pixels, mut layout) = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);

        let pixels = Pixels::new(WIDTH, HEIGHT, surface_texture)?;
        let layout = crate::layout::renderer::Layout::new(WIDTH as f32, HEIGHT as f32, body);

        (pixels, layout)
    };

    let mut now = Instant::now();

    let res = event_loop.run(|event, elwt| {
        // Draw the current frame
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            for (dst, &src) in pixels
                .frame_mut()
                .chunks_exact_mut(4)
                .zip(layout.frame().iter())
            {
                dst[0] = (src >> 16) as u8;
                dst[1] = (src >> 8) as u8;
                dst[2] = src as u8;
                dst[3] = (src >> 24) as u8;
            }

            if let Err(_) = pixels.render() {
                elwt.exit();
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
                return;
            }
            if input.scroll_diff().1 != 0.0 {
                layout.sy = (layout.sy - input.scroll_diff().1 * 20.0).max(0.0);
                layout.draw();
            }
            if input.key_held(KeyCode::ArrowDown) {
                layout.sy += 10.0;
                layout.draw();
            }
            if input.key_held(KeyCode::ArrowUp) {
                layout.sy = (layout.sy - 10.0).max(0.0);
                layout.draw();
            }
            if input.key_held(KeyCode::ArrowLeft) {
                layout.sx = (layout.sx - 10.0).max(0.0);
                layout.draw();
            }
            if input.key_held(KeyCode::ArrowRight) {
                layout.sx += 10.0;
                layout.draw();
            }
            // Resize the window
            if let Some(size) = input.window_resized() {
                layout.update_window_scale(size.width as f32, size.height as f32);
                pixels.resize_buffer(size.width, size.height).unwrap();
                pixels.resize_surface(size.width, size.height).unwrap();
                layout.draw();
            }

            // Update internal state and request a redraw
            window.request_redraw();

            now = Instant::now();
        }
    });
    res.map_err(|e| Error::UserDefined(Box::new(e)))
}
