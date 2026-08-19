//! Definicion del mapa y reglas de tiles.
//!
//! El mapa se guarda como una grilla 2D compacta. Cada celda contiene un
//! numero que representa piso, pared, obstaculo, llave, switch o salida.

use crate::player::Player;

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
            "1111111111111111111111111111111",
            "1000100000001000180010000000301",
            "1110111011301010105010111110101",
            "1010000010001010001000100010001",
            "1011111510111011111031101011101",
            "1000100010100010000010000017001",
            "1011101010101111111110111010111",
            "1010001000100000000000100010101",
            "1010111111101151131110101110101",
            "1000100000100000100010301000029",
            "1015301110111111101010101113111",
            "1010001010000000101010100010001",
            "1010111011111130101010111010101",
            "1010100000100010001010001010101",
            "1010101011501011111011103010101",
            "1010103000001000001010001000101",
            "1010111111101011101010111111501",
            "1010100000001010300010000000101",
            "1010101101101010151113111111101",
            "1010001000001000100000000030001",
            "1011115031111110111011111010111",
            "1000001000100010001000100010001",
            "1111101110101011100010105111101",
            "1000000000101000000010000000001",
            "1111111111111111111111111111111",
        ];

        let width = rows[0].len();
        let height = rows.len();
        debug_assert!(rows.iter().all(|row| row.len() == width));
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

    /// Posicion inicial de la pared perseguidora.
    pub fn chaser_spawn(&self) -> (f32, f32) {
        (27.5, 23.5)
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

    /// Indica si una celda puede ser usada por entidades que caminan.
    pub fn is_walkable_cell(&self, x: i32, y: i32) -> bool {
        !self.blocks_player(self.displayed_tile_at(x, y))
    }

    /// Aplica interacciones del jugador con tiles especiales del mapa.
    pub fn update_player_interactions(&mut self, player: &mut Player) -> Option<&'static str> {
        if self.completed {
            return None;
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
            TILE_FLOOR | TILE_KEY | TILE_SWITCH | TILE_EXIT => false,
            TILE_GATE => !self.gate_open(),
            TILE_WALL | TILE_METAL | TILE_RUINS => true,
            _ => true,
        }
    }

    /// Cambia una celda del mapa si esta dentro de limites.
    fn set_tile(&mut self, x: i32, y: i32, tile: u8) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        self.tiles[y as usize * self.width + x as usize] = tile;
    }
}
