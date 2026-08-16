use crate::{
    config::{FOV_FACTOR, MINIMAP_CELL_SIZE, MINIMAP_PADDING},
    map::{
        Map, TILE_EXIT, TILE_FLOOR, TILE_GATE, TILE_HAZARD, TILE_KEY, TILE_METAL, TILE_RUINS,
        TILE_SWITCH, TILE_WALL,
    },
    player::Player,
};

pub struct Renderer {
    width: usize,
    height: usize,
    buffer: Vec<u32>,
    depth_buffer: Vec<f32>,
}

struct RayHit {
    distance: f32,
    wall_id: u8,
    side: i32,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0; width * height],
            depth_buffer: vec![f32::INFINITY; width],
        }
    }

    pub fn buffer(&self) -> &[u32] {
        &self.buffer
    }

    pub fn render(&mut self, map: &Map, player: &Player, fps: u32, message: &str) {
        self.draw_background();
        self.draw_walls(map, player);
        self.draw_sprites(map, player);
        self.draw_minimap(map, player);
        self.draw_hud(map, player, fps, message);
    }

    fn draw_background(&mut self) {
        let horizon = self.height / 2;

        for y in 0..horizon {
            let shade = 38 + (y as u32 * 34 / horizon as u32);
            self.draw_horizontal_line(y, rgb(shade, shade + 8, shade + 18));
        }

        for y in horizon..self.height {
            let depth = (y - horizon) as u32;
            let shade = 54_u32.saturating_sub(depth * 22 / horizon as u32);
            self.draw_horizontal_line(y, rgb(shade + 20, shade + 17, shade + 12));
        }
    }

    fn draw_walls(&mut self, map: &Map, player: &Player) {
        let dir_x = player.angle.cos();
        let dir_y = player.angle.sin();
        let plane_x = -dir_y * FOV_FACTOR;
        let plane_y = dir_x * FOV_FACTOR;

        for screen_x in 0..self.width {
            let camera_x = 2.0 * screen_x as f32 / self.width as f32 - 1.0;
            let ray_dir_x = dir_x + plane_x * camera_x;
            let ray_dir_y = dir_y + plane_y * camera_x;
            let hit = cast_ray(map, player.x, player.y, ray_dir_x, ray_dir_y);
            self.depth_buffer[screen_x] = hit.distance;

            let line_height = (self.height as f32 / hit.distance.max(0.001)) as i32;
            let center_y = self.height as i32 / 2;
            let draw_start = (-line_height / 2 + center_y).max(0) as usize;
            let draw_end = (line_height / 2 + center_y).min(self.height as i32 - 1) as usize;
            let color = shade_wall(wall_color(hit.wall_id), hit.side, hit.distance);

            for y in draw_start..=draw_end {
                self.put_pixel(screen_x, y, color);
            }
        }
    }

    fn draw_sprites(&mut self, map: &Map, player: &Player) {
        let mut sprites = Vec::new();

        for y in 0..map.height() {
            for x in 0..map.width() {
                let tile = map.tile_at(x as i32, y as i32);

                if matches!(tile, TILE_HAZARD | TILE_KEY | TILE_SWITCH | TILE_EXIT) {
                    let sprite_x = x as f32 + 0.5;
                    let sprite_y = y as f32 + 0.5;
                    let distance = (sprite_x - player.x).hypot(sprite_y - player.y);
                    sprites.push((distance, sprite_x, sprite_y, tile));
                }
            }
        }

        sprites.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let dir_x = player.angle.cos();
        let dir_y = player.angle.sin();
        let plane_x = -dir_y * FOV_FACTOR;
        let plane_y = dir_x * FOV_FACTOR;
        let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);

        for (_, sprite_x, sprite_y, tile) in sprites {
            let rel_x = sprite_x - player.x;
            let rel_y = sprite_y - player.y;
            let transform_x = inv_det * (dir_y * rel_x - dir_x * rel_y);
            let transform_y = inv_det * (-plane_y * rel_x + plane_x * rel_y);

            if transform_y <= 0.05 {
                continue;
            }

            let screen_x = ((self.width as f32 / 2.0) * (1.0 + transform_x / transform_y)) as i32;
            let size_scale = if tile == TILE_HAZARD { 0.62 } else { 0.46 };
            let sprite_size = (self.height as f32 * size_scale / transform_y.abs()) as i32;
            let center_y = self.height as i32 / 2;
            let start_y = (-sprite_size / 2 + center_y).max(0);
            let end_y = (sprite_size / 2 + center_y).min(self.height as i32 - 1);
            let start_x = (-sprite_size / 2 + screen_x).max(0);
            let end_x = (sprite_size / 2 + screen_x).min(self.width as i32 - 1);

            for stripe in start_x..=end_x {
                let stripe_index = stripe as usize;

                if transform_y >= self.depth_buffer[stripe_index] {
                    continue;
                }

                for y in start_y..=end_y {
                    if let Some(color) =
                        sprite_color(tile, stripe - start_x, y - start_y, sprite_size.max(1))
                    {
                        self.put_pixel(stripe_index, y as usize, color);
                    }
                }
            }
        }
    }

    fn draw_minimap(&mut self, map: &Map, player: &Player) {
        let scale = MINIMAP_CELL_SIZE;
        let map_width = map.width() * scale;
        let map_height = map.height() * scale;
        let origin_x = self.width.saturating_sub(map_width + MINIMAP_PADDING);
        let origin_y = MINIMAP_PADDING;

        self.fill_rect(
            origin_x.saturating_sub(4),
            origin_y.saturating_sub(4),
            map_width + 8,
            map_height + 8,
            0x101319,
        );

        for y in 0..map.height() {
            for x in 0..map.width() {
                let tile = map.displayed_tile_at(x as i32, y as i32);
                let color = if tile == TILE_FLOOR {
                    0x222834
                } else {
                    minimap_color(tile)
                };

                self.fill_rect(
                    origin_x + x * scale,
                    origin_y + y * scale,
                    scale - 1,
                    scale - 1,
                    color,
                );
            }
        }

        let player_x = origin_x as i32 + (player.x * scale as f32) as i32;
        let player_y = origin_y as i32 + (player.y * scale as f32) as i32;
        self.fill_circle(player_x, player_y, 4, 0xffffff);

        let line_x = player_x + (player.angle.cos() * 12.0) as i32;
        let line_y = player_y + (player.angle.sin() * 12.0) as i32;
        self.draw_line(player_x, player_y, line_x, line_y, 0xfff36b);

        let cooldown_ratio = player.dash_cooldown_ratio();

        if cooldown_ratio > 0.0 {
            self.draw_dash_cooldown_indicator(
                (origin_x + map_width / 2) as i32,
                (origin_y + map_height + 24) as i32,
                cooldown_ratio,
            );
        }
    }

    fn draw_hud(&mut self, map: &Map, player: &Player, fps: u32, message: &str) {
        self.draw_text(12, 12, &format!("FPS {}", fps), 0xffffff, 3);
        self.draw_text(12, 34, &format!("HP {}", player.lives), 0xfff0a3, 3);

        let key_text = if map.has_key() { "KEY YES" } else { "KEY NO" };
        let gate_text = if map.gate_open() {
            "GATE OPEN"
        } else {
            "GATE SHUT"
        };

        self.draw_text(12, 60, key_text, 0xffdd57, 2);
        self.draw_text(12, 78, gate_text, 0x8ecae6, 2);
        self.draw_text(12, self.height.saturating_sub(34), message, 0xffffff, 3);
        self.draw_text(
            12,
            self.height.saturating_sub(54),
            "Q DASH  A D TURN",
            0xa9b4c4,
            2,
        );
    }

    fn draw_dash_cooldown_indicator(&mut self, center_x: i32, center_y: i32, ratio: f32) {
        self.fill_circle(center_x, center_y, 15, 0x101319);
        self.fill_clockwise_circle_slice(center_x, center_y, 12, ratio, 0xffc857);
        self.fill_circle(center_x, center_y, 7, 0x202631);
        self.fill_circle(center_x, center_y, 3, 0xffffff);
    }

    fn fill_clockwise_circle_slice(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius: i32,
        ratio: f32,
        color: u32,
    ) {
        let radius_sq = radius * radius;
        let sweep = ratio.clamp(0.0, 1.0) * std::f32::consts::TAU;

        for y in -radius..=radius {
            for x in -radius..=radius {
                if x * x + y * y > radius_sq {
                    continue;
                }

                let mut angle = (y as f32).atan2(x as f32) + std::f32::consts::FRAC_PI_2;

                if angle < 0.0 {
                    angle += std::f32::consts::TAU;
                }

                if angle <= sweep {
                    let px = center_x + x;
                    let py = center_y + y;

                    if px >= 0 && py >= 0 {
                        self.put_pixel(px as usize, py as usize, color);
                    }
                }
            }
        }
    }

    fn draw_horizontal_line(&mut self, y: usize, color: u32) {
        let row_start = y * self.width;
        let row_end = row_start + self.width;
        self.buffer[row_start..row_end].fill(color);
    }

    fn put_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            self.buffer[y * self.width + x] = color;
        }
    }

    fn fill_rect(&mut self, x: usize, y: usize, rect_width: usize, rect_height: usize, color: u32) {
        for draw_y in y..(y + rect_height).min(self.height) {
            for draw_x in x..(x + rect_width).min(self.width) {
                self.put_pixel(draw_x, draw_y, color);
            }
        }
    }

    fn fill_circle(&mut self, center_x: i32, center_y: i32, radius: i32, color: u32) {
        let radius_sq = radius * radius;

        for y in -radius..=radius {
            for x in -radius..=radius {
                if x * x + y * y <= radius_sq {
                    let px = center_x + x;
                    let py = center_y + y;

                    if px >= 0 && py >= 0 {
                        self.put_pixel(px as usize, py as usize, color);
                    }
                }
            }
        }
    }

    fn draw_line(&mut self, start_x: i32, start_y: i32, end_x: i32, end_y: i32, color: u32) {
        let mut x = start_x;
        let mut y = start_y;
        let dx = (end_x - start_x).abs();
        let sx = if start_x < end_x { 1 } else { -1 };
        let dy = -(end_y - start_y).abs();
        let sy = if start_y < end_y { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            if x >= 0 && y >= 0 {
                self.put_pixel(x as usize, y as usize, color);
            }

            if x == end_x && y == end_y {
                break;
            }

            let twice_err = 2 * err;

            if twice_err >= dy {
                err += dy;
                x += sx;
            }

            if twice_err <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn draw_text(&mut self, x: usize, y: usize, text: &str, color: u32, scale: usize) {
        let mut cursor_x = x;

        for ch in text.chars() {
            if ch == ' ' {
                cursor_x += 4 * scale;
                continue;
            }

            self.draw_char(cursor_x, y, ch, color, scale);
            cursor_x += 4 * scale;
        }
    }

    fn draw_char(&mut self, x: usize, y: usize, ch: char, color: u32, scale: usize) {
        let Some(pattern) = glyph(ch) else {
            return;
        };

        for (row, bits) in pattern.iter().enumerate() {
            for col in 0..3 {
                if bits & (1 << (2 - col)) != 0 {
                    self.fill_rect(x + col * scale, y + row * scale, scale, scale, color);
                }
            }
        }
    }
}

fn cast_ray(map: &Map, pos_x: f32, pos_y: f32, ray_dir_x: f32, ray_dir_y: f32) -> RayHit {
    let mut map_x = pos_x.floor() as i32;
    let mut map_y = pos_y.floor() as i32;
    let delta_dist_x = if ray_dir_x == 0.0 {
        f32::INFINITY
    } else {
        (1.0 / ray_dir_x).abs()
    };
    let delta_dist_y = if ray_dir_y == 0.0 {
        f32::INFINITY
    } else {
        (1.0 / ray_dir_y).abs()
    };

    let (step_x, mut side_dist_x) = if ray_dir_x < 0.0 {
        (-1, (pos_x - map_x as f32) * delta_dist_x)
    } else {
        (1, (map_x as f32 + 1.0 - pos_x) * delta_dist_x)
    };

    let (step_y, mut side_dist_y) = if ray_dir_y < 0.0 {
        (-1, (pos_y - map_y as f32) * delta_dist_y)
    } else {
        (1, (map_y as f32 + 1.0 - pos_y) * delta_dist_y)
    };

    let mut side = 0;
    let mut wall_id = 1;

    for _ in 0..128 {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            side = 0;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            side = 1;
        }

        wall_id = map.displayed_tile_at(map_x, map_y);

        if map.is_ray_blocking(map_x, map_y) {
            break;
        }
    }

    let distance = if side == 0 {
        (map_x as f32 - pos_x + (1 - step_x) as f32 / 2.0) / ray_dir_x
    } else {
        (map_y as f32 - pos_y + (1 - step_y) as f32 / 2.0) / ray_dir_y
    };

    RayHit {
        distance: distance.abs(),
        wall_id,
        side,
    }
}

fn wall_color(wall_id: u8) -> u32 {
    match wall_id {
        TILE_WALL => 0x8ecae6,
        TILE_GATE => 0xffb703,
        TILE_METAL => 0xfb8500,
        TILE_RUINS => 0xc77dff,
        _ => 0xe0e0e0,
    }
}

fn minimap_color(tile: u8) -> u32 {
    match tile {
        TILE_WALL => 0x8ecae6,
        TILE_GATE => 0xffb703,
        TILE_METAL => 0xfb8500,
        TILE_RUINS => 0xc77dff,
        TILE_HAZARD => 0xe63946,
        TILE_KEY => 0xffdd57,
        TILE_SWITCH => 0x4cc9f0,
        TILE_EXIT => 0xb7efc5,
        _ => 0xe0e0e0,
    }
}

fn sprite_color(tile: u8, local_x: i32, local_y: i32, size: i32) -> Option<u32> {
    let half = size as f32 / 2.0;
    let nx = (local_x as f32 - half) / half.max(1.0);
    let ny = (local_y as f32 - half) / half.max(1.0);

    match tile {
        TILE_KEY => {
            if nx.abs() + ny.abs() < 0.74 {
                Some(0xffdd57)
            } else if nx > 0.45 && ny.abs() < 0.18 {
                Some(0xfff3b0)
            } else {
                None
            }
        }
        TILE_SWITCH => {
            if nx.abs() < 0.68 && ny.abs() < 0.68 {
                Some(if ny < -0.2 { 0x90e0ef } else { 0x0077b6 })
            } else {
                None
            }
        }
        TILE_HAZARD => {
            if ny > -0.65 && ny < 0.72 && nx.abs() < 0.85 - ny.abs() * 0.35 {
                Some(if ny < -0.18 { 0xffb703 } else { 0xd62828 })
            } else {
                None
            }
        }
        TILE_EXIT => {
            let dist = nx.hypot(ny);

            if dist < 0.72 && dist > 0.48 {
                Some(0xb7efc5)
            } else if nx.abs() < 0.14 && ny.abs() < 0.52 {
                Some(0x52b788)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn shade_wall(color: u32, side: i32, distance: f32) -> u32 {
    let side_factor = if side == 1 { 0.72 } else { 1.0 };
    let distance_factor = (1.0 / (1.0 + distance * 0.08)).clamp(0.35, 1.0);
    let factor = side_factor * distance_factor;

    let r = (((color >> 16) & 0xff) as f32 * factor) as u32;
    let g = (((color >> 8) & 0xff) as f32 * factor) as u32;
    let b = ((color & 0xff) as f32 * factor) as u32;

    rgb(r, g, b)
}

fn rgb(r: u32, g: u32, b: u32) -> u32 {
    (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
}

fn glyph(ch: char) -> Option<[u8; 5]> {
    match ch {
        '0' => Some([0b111, 0b101, 0b101, 0b101, 0b111]),
        '1' => Some([0b010, 0b110, 0b010, 0b010, 0b111]),
        '2' => Some([0b111, 0b001, 0b111, 0b100, 0b111]),
        '3' => Some([0b111, 0b001, 0b111, 0b001, 0b111]),
        '4' => Some([0b101, 0b101, 0b111, 0b001, 0b001]),
        '5' => Some([0b111, 0b100, 0b111, 0b001, 0b111]),
        '6' => Some([0b111, 0b100, 0b111, 0b101, 0b111]),
        '7' => Some([0b111, 0b001, 0b010, 0b010, 0b010]),
        '8' => Some([0b111, 0b101, 0b111, 0b101, 0b111]),
        '9' => Some([0b111, 0b101, 0b111, 0b001, 0b111]),
        'A' => Some([0b010, 0b101, 0b111, 0b101, 0b101]),
        'B' => Some([0b110, 0b101, 0b110, 0b101, 0b110]),
        'C' => Some([0b111, 0b100, 0b100, 0b100, 0b111]),
        'D' => Some([0b110, 0b101, 0b101, 0b101, 0b110]),
        'E' => Some([0b111, 0b100, 0b110, 0b100, 0b111]),
        'F' => Some([0b111, 0b100, 0b111, 0b100, 0b100]),
        'G' => Some([0b111, 0b100, 0b101, 0b101, 0b111]),
        'H' => Some([0b101, 0b101, 0b111, 0b101, 0b101]),
        'I' => Some([0b111, 0b010, 0b010, 0b010, 0b111]),
        'J' => Some([0b001, 0b001, 0b001, 0b101, 0b111]),
        'K' => Some([0b101, 0b101, 0b110, 0b101, 0b101]),
        'L' => Some([0b100, 0b100, 0b100, 0b100, 0b111]),
        'M' => Some([0b101, 0b111, 0b111, 0b101, 0b101]),
        'N' => Some([0b101, 0b111, 0b111, 0b111, 0b101]),
        'O' => Some([0b111, 0b101, 0b101, 0b101, 0b111]),
        'P' => Some([0b110, 0b101, 0b110, 0b100, 0b100]),
        'Q' => Some([0b111, 0b101, 0b101, 0b111, 0b001]),
        'R' => Some([0b110, 0b101, 0b110, 0b101, 0b101]),
        'S' => Some([0b111, 0b100, 0b111, 0b001, 0b111]),
        'T' => Some([0b111, 0b010, 0b010, 0b010, 0b010]),
        'U' => Some([0b101, 0b101, 0b101, 0b101, 0b111]),
        'V' => Some([0b101, 0b101, 0b101, 0b101, 0b010]),
        'W' => Some([0b101, 0b101, 0b111, 0b111, 0b101]),
        'X' => Some([0b101, 0b101, 0b010, 0b101, 0b101]),
        'Y' => Some([0b101, 0b101, 0b010, 0b010, 0b010]),
        'Z' => Some([0b111, 0b001, 0b010, 0b100, 0b111]),
        _ => None,
    }
}
