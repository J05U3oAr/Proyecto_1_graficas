//! Definicion del mapa y reglas de tiles.
//!
//! Los niveles se escriben como mazes ASCII legibles. Cada caracter representa
//! piso, pared, obstaculo, spawn u objetivo, y luego se convierte a una grilla
//! compacta de tiles para colisiones, raycasting y pathfinding.

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
        let maze = [
            "+---+-------+---+---+---------+",
            "| p |       |   |s  |       m |",
            "+-+ +-- +-m | | | r | +---+ + |",
            "| |     |   | |   |   |   |   |",
            "| +-+--r| +-+ +---+ m-+ + +-- |",
            "|   |   | |   |     |     |k  |",
            "| +-+ | | | --+-----+ +-- | +-+",
            "| |   |   |           |   | | |",
            "| | +-+---+ --r-+m--+ | +-+ | |",
            "|   |     |     |   | m |    dg",
            "| |rm +-+ +-----+ | | | +-+m--+",
            "| |   | |       | | | |   |   |",
            "| | +-+ +-+---m | | | +-+ | | |",
            "| | |     |   |   | |   | | | |",
            "| | | + --r | +---+ +-- m | | |",
            "| | | m     |     | |   |   | |",
            "| | +------ | +-- | | --+---r |",
            "| | |       | | m   |       | |",
            "| | | +- -- | | |r--+m------+ |",
            "| |   |     |   |         m   |",
            "| +---r m-+-+-+ +-+ --+-- | --+",
            "|     |   |   |   |   |   |   |",
            "+---- +-- | | +--   | | r-+-- |",
            "|         | |       |      e  |",
            "+---------+-+-------+---------+",
        ];

        Self::from_maze(&maze, 0.15)
    }

    /// Construye el segundo nivel con objetivos repartidos en extremos opuestos.
    pub fn level_two() -> Self {
        let maze = [
            "+-----+-----------------------+",
            "| p   |         r             |",
            "| +r+ | m+    | + -- -- rs| + |",
            "|   r         |   m   m | |   |",
            "| + r   + r-- | mrm + --+ | + |",
            "|   |     r       |     |     |",
            "| + | + r | m --r | | | | --- |",
            "|     m   | |   |   | | |     |",
            "| ---r--+ | |r+ | +-r |   r-+ |",
            "|       | r   r   |     +   | |",
            "| m + | r + + +   r +r  r+  | |",
            "| |   |           m r       | |",
            "| +-  r-+-- | + + |   + +r -+ |",
            "| |     |   |     |          dg",
            "| |  -- |r+ r+r + +-r-- +---+r|",
            "| |             r       |   | |",
            "| |m+ +r| | + --m + + + | r | |",
            "|       | | m             |   |",
            "| r---r-+   +   + r-+ + +r| + |",
            "| |       |         |       r |",
            "| | -- +m-+---+ r+m |mm + |   |",
            "|       +     |   m   +   |e+ |",
            "| |k+     m+m +-- + | m+    r |",
            "| |                 |     |   |",
            "+-+-----------------+-----+---+",
        ];

        Self::from_maze(&maze, 0.15)
    }

    /// Construye el tercer nivel, iniciando al jugador desde el lado opuesto.
    pub fn level_three() -> Self {
        let maze = [
            "+-----------------------+-----+",
            "|                       |     |",
            "+-- +r+r|   | m   |m+ rr| --- |",
            "|  s    | | |     |           |",
            "| +m| --r | | +r|mr ---   m-- |",
            "|   |     |     |     m   +   |",
            "| +   m  m|m -+ |   +--- +r m |",
            "|   |     |   | | | |     |  dg",
            "| +-+-- | |     +-+ | + | |   |",
            "| |     |   + |   m |   |   | |",
            "| | --r   m r +-- m m   | + +-+",
            "|       +                 m   |",
            "| + + | r m + --- +   r-- --+ |",
            "|     |               k   r | |",
            "| m+r+m+m--r+ r +-  +---- r | |",
            "| +             |   |       r |",
            "|   | | | --- + | | +---+-- | |",
            "|   | | |   r   | |     |   | |",
            "+---r     |   --r | --+ +- +r |",
            "|     | + | m         |       |",
            "|  +m-+   | | +-  --r rm+m -- |",
            "|  e    | | | |     + +       |",
            "| -- --r|   |   +   m m---  + |",
            "|               m           p |",
            "+-----------------------------+",
        ];

        Self::from_maze(&maze, 3.1)
    }

    fn from_maze(rows: &[&str], player_angle: f32) -> Self {
        let width = rows
            .iter()
            .map(|row| row.chars().count())
            .max()
            .expect("maze must have at least one row");
        let height = rows.len();
        let mut tiles = vec![TILE_WALL; width * height];
        let mut player_spawn = None;
        let mut chaser_spawn = None;

        for (y, row) in rows.iter().enumerate() {
            for (x, symbol) in row.chars().enumerate() {
                let tile = match symbol {
                    ' ' => TILE_FLOOR,
                    '+' | '-' | '|' | '#' => TILE_WALL,
                    'd' | 'D' => TILE_GATE,
                    'm' | 'M' => TILE_METAL,
                    'r' | 'R' => TILE_RUINS,
                    'k' | 'K' => TILE_KEY,
                    's' | 'S' => TILE_SWITCH,
                    'g' | 'G' => TILE_EXIT,
                    'p' | 'P' => {
                        player_spawn = Some((x as f32 + 0.5, y as f32 + 0.5, player_angle));
                        TILE_FLOOR
                    }
                    'e' | 'E' | 'c' | 'C' => {
                        chaser_spawn = Some((x as f32 + 0.5, y as f32 + 0.5));
                        TILE_FLOOR
                    }
                    _ => TILE_WALL,
                };

                tiles[y * width + x] = tile;
            }
        }

        Self {
            width,
            height,
            tiles,
            has_key: false,
            switch_pressed: false,
            completed: false,
            player_spawn: player_spawn.expect("maze must include a player spawn marked with p"),
            chaser_spawn: chaser_spawn.expect("maze must include a chaser spawn marked with e"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maze_symbols_are_converted_to_tiles_and_spawns() {
        let map = Map::from_maze(&["+dm", "|pke", "|srg"], 1.25);

        assert_eq!(map.width(), 4);
        assert_eq!(map.height(), 3);
        assert_eq!(map.player_spawn(), (1.5, 1.5, 1.25));
        assert_eq!(map.chaser_spawn(), (3.5, 1.5));
        assert_eq!(map.tile_at(0, 0), TILE_WALL);
        assert_eq!(map.tile_at(1, 0), TILE_GATE);
        assert_eq!(map.tile_at(2, 0), TILE_METAL);
        assert_eq!(map.tile_at(2, 1), TILE_KEY);
        assert_eq!(map.tile_at(1, 2), TILE_SWITCH);
        assert_eq!(map.tile_at(2, 2), TILE_RUINS);
        assert_eq!(map.tile_at(3, 2), TILE_EXIT);
    }
}
