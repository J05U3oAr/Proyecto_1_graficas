//! Movimiento y estado del jugador.
//!
//! Este modulo convierte acciones de input en desplazamiento dentro del mapa,
//! aplicando colisiones, dash, vida y respawn.

use crate::{
    config::{
        COLLISION_STEP, DASH_COOLDOWN, DASH_DISTANCE, MOUSE_SENSITIVITY, MOVE_SPEED, PLAYER_RADIUS,
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
    /// Tiempo restante antes de poder usar dash otra vez.
    dash_cooldown: f32,
    /// Bandera temporal que se activa al tocar un hazard.
    touched_hazard: bool,
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
            dash_cooldown: 0.0,
            touched_hazard: false,
            spawn_x: x,
            spawn_y: y,
            spawn_angle: angle,
        }
    }

    /// Progreso normalizado del cooldown del dash, util para dibujarlo en HUD.
    pub fn dash_cooldown_ratio(&self) -> f32 {
        (self.dash_cooldown / DASH_COOLDOWN).clamp(0.0, 1.0)
    }

    /// Indica si durante este frame el jugador piso un hazard.
    pub fn touched_hazard(&self) -> bool {
        self.touched_hazard
    }

    /// Actualiza mirada con mouse, movimiento, colisiones y dash.
    pub fn update(&mut self, input: &InputState, map: &Map, dt: f32) {
        self.touched_hazard = false;
        self.dash_cooldown = (self.dash_cooldown - dt).max(0.0);
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

        // Primero se aplica el movimiento normal; luego el dash si fue presionado.
        self.move_with_collision(delta_x, delta_y, map);

        if input.dash && self.dash_cooldown <= 0.0 {
            self.move_with_collision(dir_x * DASH_DISTANCE, dir_y * DASH_DISTANCE, map);
            self.dash_cooldown = DASH_COOLDOWN;
        }
    }

    /// Aplica dano y devuelve al jugador al punto inicial.
    pub fn take_hit_and_respawn(&mut self) {
        self.lives = self.lives.saturating_sub(1).max(1);
        self.respawn();
    }

    /// Restaura posicion, angulo y estado temporal de hazard.
    pub fn respawn(&mut self) {
        self.x = self.spawn_x;
        self.y = self.spawn_y;
        self.angle = self.spawn_angle;
        self.touched_hazard = false;
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

        if map.player_touches_hazard(self.x, self.y, PLAYER_RADIUS) {
            self.touched_hazard = true;
        }
    }
}
