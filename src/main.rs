#![feature(cursor_remaining)]
#![feature(read_buf)]
#![feature(allocator_api)]

use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

pub mod runtime;
pub mod input;
pub mod rw_cursor;

use runtime::Runtime;

const WINDOW_WIDTH: u32 = 1680;
const WINDOW_HEIGHT: u32 = 720;

const BUFFER_WIDTH: u32 = 336;
const BUFFER_HEIGHT: u32 = 144;
const BUFFER_LEN: usize = BUFFER_WIDTH as usize * BUFFER_HEIGHT as usize;

const PALETTE: [[u8; 3]; 29] = [
    [  0,  0,  0], // black
    [204, 36, 29], // red
    [152,151, 26], // green
    [215,153, 33], // yellow
    [ 69,133,136], // blue
    [177, 98,134], // purple
    [104,157,106], // aqua
    [214, 93, 14], // orange

    [251, 73, 52], // light_red
    [184,187, 38], // light_green
    [250,189, 47], // light_yellow
    [131,165,152], // light_blue
    [211,134,155], // light_purple
    [142,192,124], // light_aqua
    [254,128, 25], // light_orange

    [ 40, 40, 40], // bg0
    [ 60, 56, 54], // bg1
    [ 80, 73, 69], // bg2
    [102, 92, 84], // bg3
    [124,111,100], // bg4
    [168,153,132], // gray0
    [146,131,116], // gray1

    [168,153,132], // fg4
    [189,174,147], // fg3
    [213,196,161], // fg2
    [235,219,178], // fg1
    [251,241,199], // fg0,

    [ 29, 32, 33], // bg0_hard
    [ 50, 48, 47], // bg0_soft
];

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

    let mut runtime = Runtime::new("../prism-os/target/wasm32-wasi/release/prism_os.wasm", None);
    // let mut runtime = Runtime::new("disk/bin/gfx_test.wasm", None);

    let mut previous_frame = std::time::Instant::now();
    let mut delta_s = 0.0;

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            let mut pal_buf = vec![0u8; BUFFER_LEN];
            runtime.tick(&mut pal_buf, input::input_to_u64(&input), delta_s);
            let frame = pixels.get_frame();
            for i in 0..BUFFER_LEN {
                let pal = pal_buf[i];
                frame[i*4..i*4+3].copy_from_slice(&PALETTE[pal as usize]);
                frame[i*4+3] = 255;
            }
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
