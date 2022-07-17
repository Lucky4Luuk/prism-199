use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WINDOW_WIDTH: u32 = 1680;
const WINDOW_HEIGHT: u32 = 720;

const BUFFER_WIDTH: u32 = 168;
const BUFFER_HEIGHT: u32 = 72;

struct FrameInfo<'frame> {
    buf: &'frame mut [u8],
    resolution: (usize, usize),
}

fn draw(info: FrameInfo) {
    for x in 0..info.resolution.0 {
        for y in 0..info.resolution.1 {
            let i = (x + y * info.resolution.0) * 4;
            info.buf[i  ] = 255;
            info.buf[i+1] = 0;
            info.buf[i+2] = 0;
            info.buf[i+3] = 255;
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("PRISM-199 - FANTASY COMPUTER - V[0.1]")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(BUFFER_WIDTH, BUFFER_HEIGHT, surface_texture).expect("Failed to create Pixels object!")
    };

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            draw(FrameInfo {
                buf: pixels.get_frame(),
                resolution: (BUFFER_WIDTH as usize, BUFFER_HEIGHT as usize),
            });
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
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
                // resolution = (size.width as usize, size.height as usize);
            }

            // Update internal state and request a redraw
            window.request_redraw();
        }
    });
}
