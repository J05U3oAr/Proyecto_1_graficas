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
    /// Posicion X, Y y angulo inicial del jugador.
    player_spawn: (f32, f32, f32),
    /// Posicion inicial de la pared perseguidora.
    chaser_spawn: (f32, f32),
}

impl Map {
    /// Cantidad de niveles disponibles.
    pub fn level_count() -> usize {
        3
    }

    /// Construye un nivel por indice.
    pub fn level(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::level_one()),
            1 => Some(Self::level_two()),
            2 => Some(Self::level_three()),
            _ => None,
        }
    }

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

        Self::from_rows(&rows, (2.5, 1.5, 0.15), (27.5, 23.5))
    }

    /// Construye el segundo nivel con objetivos repartidos en extremos opuestos.
    pub fn level_two() -> Self {
        let rows = [
            "1111111111111111111111111111111",
            "1000001000000000500000000000001",
            "1015101031000010101101105810101",
            "1000500000000010003000301010001",
            "1010500010511010353010111010101",
            "1000100000500000001000001000001",
            "1010101050103011501010101011101",
            "1000003000101000100010101000001",
            "1011151110101510101150100051101",
            "1000000010500050001000001000101",
            "1030101050101010005015005100101",
            "1010001000000000003050000000101",
            "1011005111101010101000101501101",
            "1010000010001000001000000000029",
            "1010011015105150101151101111151",
            "1010000000000000500000001000101",
            "1013101510101011301010101050101",
            "1000000010103000000000000010001",
            "1051115110001000105110101510101",
            "1010000000100000000010000000501",
            "1010110131111110513013301010001",
            "1000000010000010003000100010101",
            "1017100000313011101010310000501",
            "1010000000000000000010000010001",
            "1111111111111111111111111111111",
        ];

        Self::from_rows(&rows, (2.5, 1.5, 0.15), (27.5, 21.5))
    }

    /// Construye el tercer nivel, iniciando al jugador desde el lado opuesto.
    pub fn level_three() -> Self {
        let rows = [
            "1111111111111111111111111111111",
            "1000000000000000000000001000001",
            "1110151510001030001310551011101",
            "1008000010101000001000000000001",
            "1013101150101015135011100031101",
            "1000100000100000100000300010001",
            "1010003003130110100011110150301",
            "1000100000100010101010000010029",
            "1011111010100000111010101010001",
            "1010000010001010003010001000101",
            "1010115000305011103030001010111",
            "1000000010000000000000000030001",
            "1010101050301011101000511011101",
            "1000001000000000000000700050101",
            "1031513131151050110011111050101",
            "1010000000000000100010000000501",
            "1000101010111010101011111110101",
            "1000101010005000101000001000101",
            "1111500000100011501011101101501",
            "1000001010103000000000100000001",
            "1001311000101011001150531301101",
            "1000000010101010000010100000001",
            "1011011510001000100030311100101",
            "1000000000000000300000000000001",
            "1111111111111111111111111111111",
        ];

        Self::from_rows(&rows, (28.5, 23.5, 3.1), (3.5, 21.5))
    }

    fn from_rows(rows: &[&str], player_spawn: (f32, f32, f32), chaser_spawn: (f32, f32)) -> Self {
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
            player_spawn,
            chaser_spawn,
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
        self.player_spawn
    }

    /// Posicion inicial de la pared perseguidora.
    pub fn chaser_spawn(&self) -> (f32, f32) {
        self.chaser_spawn
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
            return Some("ACCESS CARD FOUND");
        }

        if tile == TILE_SWITCH {
            if self.has_key {
                self.switch_pressed = true;
                return Some("DOOR UNLOCKED");
            }

            return Some("NEED ACCESS CARD");
        }

        if tile == TILE_EXIT {
            if self.gate_open() {
                self.completed = true;
                return Some("SITE SECURED");
            }

            return Some("EXIT SEALED");
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
