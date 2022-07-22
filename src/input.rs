use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

pub fn input_to_u64(input: &WinitInputHelper) -> u64 {
    let buttons = [
        //0-25
        VirtualKeyCode::A,          //  0
        VirtualKeyCode::B,          //  1
        VirtualKeyCode::C,          //  2
        VirtualKeyCode::D,          //  3
        VirtualKeyCode::E,          //  4
        VirtualKeyCode::F,          //  5
        VirtualKeyCode::G,          //  6
        VirtualKeyCode::H,          //  7
        VirtualKeyCode::I,          //  8
        VirtualKeyCode::J,          //  9
        VirtualKeyCode::K,          // 10
        VirtualKeyCode::L,          // 11
        VirtualKeyCode::M,          // 12
        VirtualKeyCode::N,          // 13
        VirtualKeyCode::O,          // 14
        VirtualKeyCode::P,          // 15
        VirtualKeyCode::Q,          // 16
        VirtualKeyCode::R,          // 17
        VirtualKeyCode::S,          // 18
        VirtualKeyCode::T,          // 19
        VirtualKeyCode::U,          // 20
        VirtualKeyCode::V,          // 21
        VirtualKeyCode::W,          // 22
        VirtualKeyCode::X,          // 23
        VirtualKeyCode::Y,          // 24
        VirtualKeyCode::Z,          // 25

        // 26-35
        VirtualKeyCode::Key0,       // 26
        VirtualKeyCode::Key1,       // 27
        VirtualKeyCode::Key2,       // 28
        VirtualKeyCode::Key3,       // 29
        VirtualKeyCode::Key4,       // 30
        VirtualKeyCode::Key5,       // 31
        VirtualKeyCode::Key6,       // 32
        VirtualKeyCode::Key7,       // 33
        VirtualKeyCode::Key8,       // 34
        VirtualKeyCode::Key9,       // 35

        // 36-46
        VirtualKeyCode::Minus,      // 36
        VirtualKeyCode::Plus,       // 37
        VirtualKeyCode::Equals,     // 38
        VirtualKeyCode::LBracket,   // 39
        VirtualKeyCode::RBracket,   // 40
        VirtualKeyCode::Period,     // 41
        VirtualKeyCode::Comma,      // 42
        VirtualKeyCode::Colon,      // 43
        VirtualKeyCode::Semicolon,  // 44
        VirtualKeyCode::Apostrophe, // 45
        VirtualKeyCode::Backslash,  // 46

        // 47-51
        VirtualKeyCode::Tab,        // 47
        VirtualKeyCode::Escape,     // 48
        VirtualKeyCode::Space,      // 49
        VirtualKeyCode::Back,       // 50
        VirtualKeyCode::Delete,     // 51
        VirtualKeyCode::Return,     // 52
    ];

    let mut bitfield = 0;
    for (i, btn) in buttons.iter().enumerate() {
        bitfield |= (input.key_held(*btn) as u64) << i;
    }
    bitfield
}
