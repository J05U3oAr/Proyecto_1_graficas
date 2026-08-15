use crate::{
    config::{MOVE_SPEED, PLAYER_RADIUS, ROTATION_SPEED},
    input::InputState,
    map::Map,
};

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
}

impl Player {
    pub fn new(x: f32, y: f32, angle: f32) -> Self {
        Self { x, y, angle }
    }

    pub fn update(&mut self, input: &InputState, map: &Map, dt: f32) {
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

        self.try_move(delta_x, delta_y, map);
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
    }
}
