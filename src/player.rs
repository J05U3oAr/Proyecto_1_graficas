use crate::{
    config::{
        COLLISION_STEP, DASH_COOLDOWN, DASH_DISTANCE, MOVE_SPEED, PLAYER_RADIUS, ROTATION_SPEED,
    },
    input::InputState,
    map::Map,
};

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub lives: u8,
    dash_cooldown: f32,
    touched_hazard: bool,
    spawn_x: f32,
    spawn_y: f32,
    spawn_angle: f32,
}

impl Player {
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

    pub fn dash_cooldown_ratio(&self) -> f32 {
        (self.dash_cooldown / DASH_COOLDOWN).clamp(0.0, 1.0)
    }

    pub fn touched_hazard(&self) -> bool {
        self.touched_hazard
    }

    pub fn update(&mut self, input: &InputState, map: &Map, dt: f32) {
        self.touched_hazard = false;
        self.dash_cooldown = (self.dash_cooldown - dt).max(0.0);
        let rotation_step = ROTATION_SPEED * dt;

        if input.rotate_left {
            self.angle -= rotation_step;
        }

        if input.rotate_right {
            self.angle += rotation_step;
        }

        let dir_x = self.angle.cos();
        let dir_y = self.angle.sin();
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
            delta_x += side_x * movement_step;
            delta_y += side_y * movement_step;
        }

        if input.strafe_right {
            delta_x -= side_x * movement_step;
            delta_y -= side_y * movement_step;
        }

        self.move_with_collision(delta_x, delta_y, map);

        if input.dash && self.dash_cooldown <= 0.0 {
            self.move_with_collision(dir_x * DASH_DISTANCE, dir_y * DASH_DISTANCE, map);
            self.dash_cooldown = DASH_COOLDOWN;
        }
    }

    pub fn take_hit_and_respawn(&mut self) {
        self.lives = self.lives.saturating_sub(1).max(1);
        self.respawn();
    }

    pub fn respawn(&mut self) {
        self.x = self.spawn_x;
        self.y = self.spawn_y;
        self.angle = self.spawn_angle;
        self.touched_hazard = false;
    }

    fn move_with_collision(&mut self, delta_x: f32, delta_y: f32, map: &Map) {
        let distance = delta_x.hypot(delta_y);
        let steps = (distance / COLLISION_STEP).ceil().max(1.0) as usize;
        let step_x = delta_x / steps as f32;
        let step_y = delta_y / steps as f32;

        for _ in 0..steps {
            self.try_move(step_x, step_y, map);
        }
    }

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
