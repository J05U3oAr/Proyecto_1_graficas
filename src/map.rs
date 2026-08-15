pub struct Map {
    width: usize,
    height: usize,
    tiles: Vec<u8>,
}

impl Map {
    pub fn level_one() -> Self {
        let rows = [
            "1111111111111111",
            "1000000000000001",
            "1022200003333001",
            "1000200003000001",
            "1000200003000401",
            "1000200003000401",
            "1000000000000401",
            "1000555500000401",
            "1000000500000001",
            "1444000500222001",
            "1004000000200001",
            "1004003330205001",
            "1000000000005001",
            "1000022220005001",
            "1000000000000001",
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
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn tile_at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return 1;
        }

        self.tiles[y as usize * self.width + x as usize]
    }

    pub fn is_wall(&self, x: i32, y: i32) -> bool {
        self.tile_at(x, y) != 0
    }

    pub fn can_stand_at(&self, x: f32, y: f32, radius: f32) -> bool {
        let checks = [
            (x - radius, y - radius),
            (x + radius, y - radius),
            (x - radius, y + radius),
            (x + radius, y + radius),
        ];

        checks
            .iter()
            .all(|(check_x, check_y)| !self.is_wall(check_x.floor() as i32, check_y.floor() as i32))
    }
}
