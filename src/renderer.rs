use crate::{
    config::{FOV_FACTOR, JUMP_CAMERA_SCALE, MINIMAP_CELL_SIZE, MINIMAP_PADDING},
    map::Map,
    player::Player,
};

pub struct Renderer {
    width: usize,
    height: usize,
    buffer: Vec<u32>,
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
        }
    }

    pub fn buffer(&self) -> &[u32] {
        &self.buffer
    }

    pub fn render(&mut self, map: &Map, player: &Player, fps: u32) {
        let camera_offset = (player.height * JUMP_CAMERA_SCALE) as i32;

        self.draw_background(camera_offset);
        self.draw_walls(map, player);
        self.draw_minimap(map, player);
        self.draw_text(12, 12, &format!("FPS {}", fps), 0xffffff, 3);
    }

    fn draw_background(&mut self, camera_offset: i32) {
        let horizon =
            (self.height as i32 / 2 + camera_offset).clamp(1, self.height as i32 - 1) as usize;

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
            let line_height = (self.height as f32 / hit.distance.max(0.001)) as i32;
            let camera_offset = (player.height * JUMP_CAMERA_SCALE) as i32;
            let center_y = self.height as i32 / 2 + camera_offset;
            let draw_start = (-line_height / 2 + center_y).max(0) as usize;
            let draw_end = (line_height / 2 + center_y).min(self.height as i32 - 1) as usize;
            let color = shade_wall(wall_color(hit.wall_id), hit.side, hit.distance);

            for y in draw_start..=draw_end {
                self.put_pixel(screen_x, y, color);
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
                let tile = map.tile_at(x as i32, y as i32);
                let color = if tile == 0 {
                    0x222834
                } else {
                    wall_color(tile)
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

        wall_id = map.tile_at(map_x, map_y);

        if wall_id != 0 {
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
        1 => 0x8ecae6,
        2 => 0xffb703,
        3 => 0xfb8500,
        4 => 0x90be6d,
        5 => 0xc77dff,
        _ => 0xe0e0e0,
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
        'F' => Some([0b111, 0b100, 0b111, 0b100, 0b100]),
        'P' => Some([0b110, 0b101, 0b110, 0b100, 0b100]),
        'S' => Some([0b111, 0b100, 0b111, 0b001, 0b111]),
        _ => None,
    }
}
