//! Lectura de teclado y mouse.
//!
//! Este modulo traduce teclas concretas y movimiento horizontal del mouse a
//! acciones de juego. Asi el jugador no necesita conocer la libreria de ventana.

use minifb::{Key, KeyRepeat, Window};

/// Estado de acciones que estan activas durante el frame actual.
#[derive(Default)]
pub struct InputState {
    /// Avanzar en la direccion en la que mira el jugador.
    pub move_forward: bool,
    /// Retroceder en direccion opuesta a la mirada.
    pub move_backward: bool,
    /// Movimiento horizontal del mouse usado para girar la camara.
    pub mouse_delta_x: f32,
    /// Desplazarse lateralmente hacia la izquierda.
    pub strafe_left: bool,
    /// Desplazarse lateralmente hacia la derecha.
    pub strafe_right: bool,
    /// Habilidad sonora de un solo pulso; no se mantiene al dejar la tecla presionada.
    pub sound_ability: bool,
}

impl InputState {
    /// Construye el estado de entrada leyendo la ventana actual.
    pub fn from_window(window: &Window, mouse_delta_x: f32) -> Self {
        Self {
            move_forward: window.is_key_down(Key::W) || window.is_key_down(Key::Up),
            move_backward: window.is_key_down(Key::S) || window.is_key_down(Key::Down),
            mouse_delta_x,
            strafe_left: window.is_key_down(Key::A)
                || window.is_key_down(Key::Left)
                || window.is_key_down(Key::Z),
            strafe_right: window.is_key_down(Key::D)
                || window.is_key_down(Key::Right)
                || window.is_key_down(Key::C),
            sound_ability: window.is_key_pressed(Key::Q, KeyRepeat::No),
        }
    }
}
