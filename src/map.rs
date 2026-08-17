//! Definicion del mapa y reglas de tiles.
//!
//! El mapa se guarda como una grilla 2D compacta. Cada celda contiene un
//! numero que representa piso, pared, obstaculo, llave, switch o salida.

use crate::{config::PLAYER_RADIUS, player::Player};

/// Celda vacia por la que el jugador puede caminar.
pub const TILE_FLOOR: u8 = 0;
/// Pared solida normal.
pub const TILE_WALL: u8 = 1;
/// Puerta que se abre cuando el jugador tiene llave y presiona el switch.
pub const TILE_GATE: u8 = 2;
/// Obstaculo/pared con material metalico.
pub const TILE_METAL: u8 = 3;
/// Obstaculo/pared con material de ruinas.
pub const TILE_RUINS: u8 = 5;
/// Peligro que hace respawn al jugador.
pub const TILE_HAZARD: u8 = 6;
/// Llave que desbloquea la posibilidad de abrir la puerta.
pub const TILE_KEY: u8 = 7;
/// Interruptor que abre la puerta si ya se obtuvo la llave.
pub const TILE_SWITCH: u8 = 8;
/// Meta del nivel.
pub const TILE_EXIT: u8 = 9;

/// Estado del mapa y de los objetivos del nivel.
pub struct Map {
    /// Ancho de la grilla en celdas.
    width: usize,
    /// Alto de la grilla en celdas.
    height: usize,
    /// Tiles guardados en orden fila por fila.
    tiles: Vec<u8>,
    /// Indica si la llave ya fue recogida.
    has_key: bool,
    /// Indica si el switch ya fue activado.
    switch_pressed: bool,
    /// Indica si el jugador completo el nivel.
    completed: bool,
}

impl Map {
    /// Construye el primer nivel desde texto ASCII.
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
        // Convierte cada caracter numerico del mapa en su valor `u8`.
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

    /// Ancho del mapa en celdas.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Alto del mapa en celdas.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Posicion y angulo inicial del jugador.
    pub fn player_spawn(&self) -> (f32, f32, f32) {
        (2.5, 1.5, 0.15)
    }

    /// Devuelve si el jugador ya recogio la llave.
    pub fn has_key(&self) -> bool {
        self.has_key
    }

    /// Devuelve si el switch ya fue presionado correctamente.
    pub fn switch_pressed(&self) -> bool {
        self.switch_pressed
    }

    /// La puerta queda abierta solo si se tiene llave y switch activado.
    pub fn gate_open(&self) -> bool {
        self.has_key && self.switch_pressed
    }

    /// Devuelve si el nivel ya fue completado.
    pub fn completed(&self) -> bool {
        self.completed
    }

    /// Lee el tile en una celda. Fuera del mapa cuenta como pared.
    pub fn tile_at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return TILE_WALL;
        }

        self.tiles[y as usize * self.width + x as usize]
    }

    /// Tile que debe usarse para dibujar y bloquear segun el estado actual.
    pub fn displayed_tile_at(&self, x: i32, y: i32) -> u8 {
        let tile = self.tile_at(x, y);

        if tile == TILE_GATE && self.gate_open() {
            TILE_FLOOR
        } else {
            tile
        }
    }

    /// Indica si un rayo de vision debe detenerse en esta celda.
    pub fn is_ray_blocking(&self, x: i32, y: i32) -> bool {
        matches!(
            self.displayed_tile_at(x, y),
            TILE_WALL | TILE_GATE | TILE_METAL | TILE_RUINS
        )
    }

    /// Revisa si el jugador puede ocupar una posicion circular.
    pub fn can_stand_at(&self, x: f32, y: f32, radius: f32) -> bool {
        // Se prueban las esquinas del radio para evitar cruzar paredes en diagonal.
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

    /// Devuelve si el radio del jugador toca algun hazard.
    pub fn player_touches_hazard(&self, x: f32, y: f32, radius: f32) -> bool {
        self.player_touches_tile(x, y, radius, TILE_HAZARD)
    }

    /// Aplica interacciones del jugador con tiles especiales del mapa.
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

    /// Regla de colision fisica para cada tipo de tile.
    fn blocks_player(&self, tile: u8) -> bool {
        match tile {
            TILE_FLOOR | TILE_HAZARD | TILE_KEY | TILE_SWITCH | TILE_EXIT => false,
            TILE_GATE => !self.gate_open(),
            TILE_WALL | TILE_METAL | TILE_RUINS => true,
            _ => true,
        }
    }

    /// Revisa varios puntos alrededor del jugador contra un tile objetivo.
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

    /// Cambia una celda del mapa si esta dentro de limites.
    fn set_tile(&mut self, x: i32, y: i32, tile: u8) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        self.tiles[y as usize * self.width + x as usize] = tile;
    }
}
