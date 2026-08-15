use minifb::{Key, KeyRepeat, Window};

#[derive(Default)]
pub struct InputState {
    pub move_forward: bool,
    pub move_backward: bool,
    pub rotate_left: bool,
    pub rotate_right: bool,
    pub strafe_left: bool,
    pub strafe_right: bool,
    pub jump: bool,
    pub dash: bool,
}

impl InputState {
    pub fn from_window(window: &Window) -> Self {
        Self {
            move_forward: window.is_key_down(Key::W) || window.is_key_down(Key::Up),
            move_backward: window.is_key_down(Key::S) || window.is_key_down(Key::Down),
            rotate_left: window.is_key_down(Key::A) || window.is_key_down(Key::Left),
            rotate_right: window.is_key_down(Key::D) || window.is_key_down(Key::Right),
            strafe_left: window.is_key_down(Key::Z),
            strafe_right: window.is_key_down(Key::C),
            jump: window.is_key_pressed(Key::Space, KeyRepeat::No),
            dash: window.is_key_pressed(Key::Q, KeyRepeat::No),
        }
    }
}
