//! Movimiento y estado del jugador.
//!
//! Este modulo convierte acciones de input en desplazamiento dentro del mapa,
//! aplicando colisiones, habilidad sonora, vida y respawn.

use crate::{
    config::{
        COLLISION_STEP, MOUSE_SENSITIVITY, MOVE_SPEED, PLAYER_RADIUS, SOUND_ABILITY_COOLDOWN,
    },
    input::InputState,
    map::Map,
};

/// Jugador dentro del mundo 2D del raycaster.
pub struct Player {
    /// Posicion X en coordenadas de mapa.
    pub x: f32,
    /// Posicion Y en coordenadas de mapa.
    pub y: f32,
    /// Angulo de mirada en radianes.
    pub angle: f32,
    /// Vidas actuales.
    pub lives: u8,
    /// Tiempo restante antes de poder usar la habilidad sonora otra vez.
    sound_ability_cooldown: f32,
    /// Posicion X de respawn.
    spawn_x: f32,
    /// Posicion Y de respawn.
    spawn_y: f32,
    /// Angulo de respawn.
    spawn_angle: f32,
}

impl Player {
    /// Crea un jugador en la posicion y angulo indicados.
    pub fn new(x: f32, y: f32, angle: f32) -> Self {
        Self {
            x,
            y,
            angle,
            lives: 3,
            sound_ability_cooldown: 0.0,
            spawn_x: x,
            spawn_y: y,
            spawn_angle: angle,
        }
    }

    /// Progreso normalizado del cooldown de la habilidad sonora, util para dibujarlo en HUD.
    pub fn sound_ability_cooldown_ratio(&self) -> f32 {
        (self.sound_ability_cooldown / SOUND_ABILITY_COOLDOWN).clamp(0.0, 1.0)
    }

    /// Actualiza mirada con mouse, movimiento, colisiones y habilidad sonora.
    /// Devuelve `true` si la habilidad se activo durante este frame.
    pub fn update(&mut self, input: &InputState, map: &Map, dt: f32) -> bool {
        self.sound_ability_cooldown = (self.sound_ability_cooldown - dt).max(0.0);
        self.angle += input.mouse_delta_x * MOUSE_SENSITIVITY;

        let dir_x = self.angle.cos();
        let dir_y = self.angle.sin();
        // Vector perpendicular a la mirada, usado para strafe lateral.
        let side_x = -dir_y;
        let side_y = dir_x;
        let mut delta_x = 0.0;
        let mut delta_y = 0.0;
        let movement_step = MOVE_SPEED * dt;

        if input.move_forward {
            delta_x += dir_x * movement_step;
            delta_y += dir_y * movement_step;
        }

        if input.move_backward {
            delta_x -= dir_x * movement_step;
            delta_y -= dir_y * movement_step;
        }

        if input.strafe_left {
            delta_x -= side_x * movement_step;
            delta_y -= side_y * movement_step;
        }

        if input.strafe_right {
            delta_x += side_x * movement_step;
            delta_y += side_y * movement_step;
        }

        self.move_with_collision(delta_x, delta_y, map);

        if input.sound_ability && self.sound_ability_cooldown <= 0.0 {
            self.sound_ability_cooldown = SOUND_ABILITY_COOLDOWN;
            return true;
        }

        false
    }

    /// Aplica dano y devuelve al jugador al punto inicial.
    pub fn take_hit_and_respawn(&mut self) {
        self.lives = self.lives.saturating_sub(1);

        if self.lives > 0 {
            self.respawn();
        }
    }

    /// Restaura posicion, angulo y estado temporal de hazard.
    pub fn respawn(&mut self) {
        self.x = self.spawn_x;
        self.y = self.spawn_y;
        self.angle = self.spawn_angle;
    }

    /// Divide un movimiento grande en pasos pequenos para mejorar colisiones.
    fn move_with_collision(&mut self, delta_x: f32, delta_y: f32, map: &Map) {
        let distance = delta_x.hypot(delta_y);
        let steps = (distance / COLLISION_STEP).ceil().max(1.0) as usize;
        let step_x = delta_x / steps as f32;
        let step_y = delta_y / steps as f32;

        for _ in 0..steps {
            self.try_move(step_x, step_y, map);
        }
    }

    /// Intenta mover en X y Y por separado para permitir deslizamiento en paredes.
    fn try_move(&mut self, delta_x: f32, delta_y: f32, map: &Map) {
        let next_x = self.x + delta_x;

        if map.can_stand_at(next_x, self.y, PLAYER_RADIUS) {
            self.x = next_x;
        }

        let next_y = self.y + delta_y;

        if map.can_stand_at(self.x, next_y, PLAYER_RADIUS) {
            self.y = next_y;
        }
    }
}
