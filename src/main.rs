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

const PALETTE: [[u8; 3]; 76] = [
	[9,9,14],
	[26,28,38],
	[60,72,81],
	[97,112,119],
	[153,165,167],
	[203,210,217],
	[255,255,255],
	[33,27,33],
	[107,88,83],
	[171,104,76],
	[206,166,95],
	[231,212,148],
	[249,243,192],
	[36,19,29],
	[64,40,48],
	[94,66,60],
	[128,99,82],
	[161,148,119],
	[189,187,147],
	[76,7,23],
	[129,11,11],
	[168,43,18],
	[212,92,29],
	[227,133,36],
	[235,171,76],
	[241,194,86],
	[246,221,122],
	[3,18,31],
	[15,52,63],
	[26,85,86],
	[44,125,99],
	[75,162,69],
	[148,204,71],
	[234,242,87],
	[2,16,23],
	[11,59,68],
	[23,117,110],
	[48,163,135],
	[80,205,144],
	[106,226,145],
	[201,232,161],
	[23,9,46],
	[21,21,86],
	[17,63,130],
	[52,102,176],
	[113,181,219],
	[158,228,239],
	[209,251,240],
	[38,22,70],
	[85,45,114],
	[136,75,147],
	[172,108,162],
	[197,143,170],
	[223,178,198],
	[237,209,214],
	[20,3,51],
	[70,21,101],
	[123,37,132],
	[169,75,132],
	[208,116,130],
	[222,158,140],
	[123,13,105],
	[164,16,87],
	[195,67,92],
	[225,118,118],
	[243,191,173],
	[60,19,59],
	[107,46,90],
	[170,85,124],
	[202,134,122],
	[242,205,170],
	[250,248,219],
	[138,64,40],
	[179,121,77],
	[218,181,128],
	[243,231,168],
];
const PALETTE_SIZE: usize = 76;

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
