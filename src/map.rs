use crate::{config::PLAYER_RADIUS, player::Player};

pub const TILE_FLOOR: u8 = 0;
pub const TILE_WALL: u8 = 1;
pub const TILE_GATE: u8 = 2;
pub const TILE_METAL: u8 = 3;
pub const TILE_RUINS: u8 = 5;
pub const TILE_HAZARD: u8 = 6;
pub const TILE_KEY: u8 = 7;
pub const TILE_SWITCH: u8 = 8;
pub const TILE_EXIT: u8 = 9;

pub struct Map {
    width: usize,
    height: usize,
    tiles: Vec<u8>,
    has_key: bool,
    switch_pressed: bool,
    completed: bool,
}

impl Map {
    pub fn level_one() -> Self {
        let rows = [
            "1111111111111111",
            "1000000000000001",
            "1070000666000001",
            "1000110000011001",
            "1000010000001001",
            "1000010666601001",
            "1000010000001001",
            "1000011111101001",
            "1000000000101001",
            "1111006660101001",
            "1001000000108001",
            "1001001111101111",
            "1001000000002001",
            "1000666600002001",
            "1000000000009001",
            "1111111111111111",
        ];

        let width = rows[0].len();
        let height = rows.len();
        let tiles = rows
            .iter()
            .flat_map(|row| row.bytes().map(|value| value - b'0'))
            .collect();

        Self {
            width,
            height,
            tiles,
            has_key: false,
            switch_pressed: false,
            completed: false,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn player_spawn(&self) -> (f32, f32, f32) {
        (2.5, 1.5, 0.15)
    }

    pub fn has_key(&self) -> bool {
        self.has_key
    }

    pub fn switch_pressed(&self) -> bool {
        self.switch_pressed
    }

    pub fn gate_open(&self) -> bool {
        self.has_key && self.switch_pressed
    }

    pub fn completed(&self) -> bool {
        self.completed
    }

    pub fn tile_at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return TILE_WALL;
        }

        self.tiles[y as usize * self.width + x as usize]
    }

    pub fn displayed_tile_at(&self, x: i32, y: i32) -> u8 {
        let tile = self.tile_at(x, y);

        if tile == TILE_GATE && self.gate_open() {
            TILE_FLOOR
        } else {
            tile
        }
    }

    pub fn is_ray_blocking(&self, x: i32, y: i32) -> bool {
        matches!(
            self.displayed_tile_at(x, y),
            TILE_WALL | TILE_GATE | TILE_METAL | TILE_RUINS
        )
    }

    pub fn can_stand_at(&self, x: f32, y: f32, radius: f32) -> bool {
        let checks = [
            (x - radius, y - radius),
            (x + radius, y - radius),
            (x - radius, y + radius),
            (x + radius, y + radius),
        ];

        checks.iter().all(|(check_x, check_y)| {
            let tile = self.displayed_tile_at(check_x.floor() as i32, check_y.floor() as i32);
            !self.blocks_player(tile)
        })
    }

    pub fn player_touches_hazard(&self, x: f32, y: f32, radius: f32) -> bool {
        self.player_touches_tile(x, y, radius, TILE_HAZARD)
    }

    pub fn update_player_interactions(&mut self, player: &mut Player) -> Option<&'static str> {
        if self.completed {
            return None;
        }

        if self.player_touches_hazard(player.x, player.y, PLAYER_RADIUS) {
            player.take_hit_and_respawn();
            return Some("SPIKES HIT");
        }

        let tile_x = player.x.floor() as i32;
        let tile_y = player.y.floor() as i32;
        let tile = self.tile_at(tile_x, tile_y);

        if tile == TILE_KEY {
            self.set_tile(tile_x, tile_y, TILE_FLOOR);
            self.has_key = true;
            return Some("KEY FOUND");
        }

        if tile == TILE_SWITCH {
            if self.has_key {
                self.switch_pressed = true;
                return Some("GATE OPEN");
            }

            return Some("FIND KEY FIRST");
        }

        if tile == TILE_EXIT {
            if self.gate_open() {
                self.completed = true;
                return Some("LEVEL COMPLETE");
            }

            return Some("EXIT LOCKED");
        }

        None
    }

    fn blocks_player(&self, tile: u8) -> bool {
        match tile {
            TILE_FLOOR | TILE_HAZARD | TILE_KEY | TILE_SWITCH | TILE_EXIT => false,
            TILE_GATE => !self.gate_open(),
            TILE_WALL | TILE_METAL | TILE_RUINS => true,
            _ => true,
        }
    }

    fn player_touches_tile(&self, x: f32, y: f32, radius: f32, target_tile: u8) -> bool {
        let checks = [
            (x, y),
            (x - radius, y - radius),
            (x + radius, y - radius),
            (x - radius, y + radius),
            (x + radius, y + radius),
        ];

        checks.iter().any(|(check_x, check_y)| {
            self.tile_at(check_x.floor() as i32, check_y.floor() as i32) == target_tile
        })
    }

    fn set_tile(&mut self, x: i32, y: i32, tile: u8) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        self.tiles[y as usize * self.width + x as usize] = tile;
    }
}
