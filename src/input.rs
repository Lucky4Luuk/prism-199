use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

pub fn input_to_u64(input: &WinitInputHelper) -> u64 {
    let buttons = vec![
    VirtualKeyCode::A,
    VirtualKeyCode::B,
    VirtualKeyCode::C,
    VirtualKeyCode::D,
    VirtualKeyCode::E,
    VirtualKeyCode::F,
    VirtualKeyCode::G,
    VirtualKeyCode::H,
    VirtualKeyCode::I,
    VirtualKeyCode::J,
    VirtualKeyCode::K,
    VirtualKeyCode::L,
    VirtualKeyCode::M,
    VirtualKeyCode::N,
    VirtualKeyCode::O,
    VirtualKeyCode::P,
    VirtualKeyCode::Q,
    VirtualKeyCode::R,
    VirtualKeyCode::S,
    VirtualKeyCode::T,
    VirtualKeyCode::U,
    VirtualKeyCode::V,
    VirtualKeyCode::W,
    VirtualKeyCode::X,
    VirtualKeyCode::Y,
    VirtualKeyCode::Z,

    ];

    let mut bitfield = 0;
    for (i, btn) in buttons.iter().enumerate() {
        bitfield |= (input.key_held(*btn) as u64) >> i;
    }
    bitfield
}
