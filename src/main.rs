use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

pub mod runtime;
pub mod input;

use runtime::Runtime;

const WINDOW_WIDTH: u32 = 1680;
const WINDOW_HEIGHT: u32 = 720;

const BUFFER_WIDTH: u32 = 336;
const BUFFER_HEIGHT: u32 = 144;
const BUFFER_LEN: usize = BUFFER_WIDTH as usize * BUFFER_HEIGHT as usize * 4;

pub struct FrameInfo<'frame> {
    buf: &'frame mut [u8],
}

fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);
        let min_size = LogicalSize::new(BUFFER_WIDTH as f64, BUFFER_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("PRISM-199 - FANTASY COMPUTER - V[0.1]")
            .with_inner_size(size)
            .with_min_inner_size(min_size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(BUFFER_WIDTH, BUFFER_HEIGHT, surface_texture).expect("Failed to create Pixels object!")
    };

    let mut runtime = Runtime::new("../prism-os/target/wasm32-wasi/release/prism_os.wasm");

    let mut previous_frame = std::time::Instant::now();
    let mut delta_s = 0.0;

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            runtime.tick(pixels.get_frame(), input::input_to_u64(&input), delta_s);
            if pixels
                .render()
                .map_err(|e| eprintln!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            let now = std::time::Instant::now();
            delta_s = (now - previous_frame).as_secs_f32();
            previous_frame = now;
            window.request_redraw();
        }
    });
}
