//! Renderizado del juego.
//!
//! Este modulo dibuja directamente sobre un buffer de pixeles. Incluye el
//! raycaster de paredes, sprites billboard, minimapa, HUD y utilidades basicas
//! para pintar formas/texto.

use std::path::{Path, PathBuf};

use crate::{
    chaser::Chaser,
    config::{FOV_FACTOR, MINIMAP_CELL_SIZE, MINIMAP_PADDING},
    map::{
        Map, TILE_EXIT, TILE_FLOOR, TILE_GATE, TILE_KEY, TILE_METAL, TILE_RUINS, TILE_SWITCH,
        TILE_WALL,
    },
    player::Player,
    texture::{TEXTURE_SIZE, wall_texel},
};

const SPRITE_CHASER_IDLE: u8 = 10;
const SPRITE_CHASER_ACTIVE: u8 = 11;
const SPRITE_CHASER_DISGUSTED: u8 = 12;

/// Renderer basado en un buffer de pixeles RGB.
pub struct Renderer {
    /// Ancho del buffer.
    width: usize,
    /// Alto del buffer.
    height: usize,
    /// Pixeles RGB que se envian a `minifb`.
    buffer: Vec<u32>,
    /// Fondo precalculado de cielo y piso.
    background: Vec<u32>,
    /// Distancia de pared por columna, usada para ocultar sprites detras.
    depth_buffer: Vec<f32>,
    /// Imagen opcional usada como sprite del perseguidor.
    chaser_texture: Option<SpriteTexture>,
    /// Imagen opcional usada cuando el perseguidor esta disgustado.
    disgust_chaser_texture: Option<SpriteTexture>,
}

/// Textura RGB cargada desde un archivo de imagen.
struct SpriteTexture {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
}

/// Resultado de lanzar un rayo contra el mapa.
struct RayHit {
    /// Distancia perpendicular hasta la pared.
    distance: f32,
    /// Tile golpeado por el rayo.
    wall_id: u8,
    /// Lado golpeado: 0 para eje X, 1 para eje Y.
    side: i32,
    /// Coordenada horizontal dentro de la textura.
    texture_x: usize,
}

impl Renderer {
    /// Crea un renderer con buffer de color y buffer de profundidad.
    pub fn new(width: usize, height: usize) -> Self {
        let background = build_background(width, height);

        Self {
            width,
            height,
            buffer: background.clone(),
            background,
            depth_buffer: vec![f32::INFINITY; width],
            chaser_texture: load_chaser_texture(
                "chaser",
                &["chaser.png", "chaser.jpg", "chaser.jpeg"],
            ),
            disgust_chaser_texture: load_chaser_texture("disgust chaser", &["disgust_chase.png"]),
        }
    }

    /// Expone el buffer para que la ventana pueda mostrarlo.
    pub fn buffer(&self) -> &[u32] {
        &self.buffer
    }

    /// Dibuja un frame completo en el orden correcto de capas.
    pub fn render(&mut self, map: &Map, player: &Player, chaser: &Chaser, fps: u32, message: &str) {
        self.buffer.copy_from_slice(&self.background);
        self.draw_walls(map, player);
        self.draw_sprites(map, player, chaser);
        self.draw_minimap(map, player, chaser);
        self.draw_hud(map, player, chaser, fps, message);
    }

    /// Dibuja el menu principal con la opcion actualmente seleccionada.
    pub fn render_main_menu(&mut self, selected_index: usize) {
        self.draw_menu_background();

        self.draw_centered_text(74, "RUN FROM YE", 0xf5f3e7, 10);
        self.draw_centered_text(140, "ESCAPA ANTES DE QUE LA PARED DESPIERTE", 0x9db4c4, 3);

        let options = ["JUGAR", "INSTRUCCIONES", "DE QUE TRATA", "SALIR"];
        let button_width = 360;
        let button_height = 44;
        let button_x = self.width.saturating_sub(button_width) / 2;
        let mut button_y = 208;

        for (index, label) in options.iter().enumerate() {
            self.draw_menu_button(
                button_x,
                button_y,
                button_width,
                button_height,
                label,
                index == selected_index,
            );
            button_y += 58;
        }

        self.draw_centered_text(
            self.height.saturating_sub(42),
            "W S O FLECHAS MOVER    ENTER ELEGIR    ESC SALIR",
            0x7f8c95,
            2,
        );
    }

    /// Dibuja la seleccion de nivel disponible antes de iniciar partida.
    pub fn render_level_select_screen(&mut self, selected_index: usize, level_count: usize) {
        self.draw_menu_background();

        self.draw_centered_text(78, "SELECT LEVEL", 0xf5f3e7, 7);
        self.draw_centered_text(136, "CHOOSE CONTAINMENT ROUTE", 0x9db4c4, 3);

        let button_width = 340;
        let button_height = 44;
        let button_x = self.width.saturating_sub(button_width) / 2;
        let mut button_y = 210;

        for index in 0..level_count {
            let label = format!("LEVEL {}", index + 1);
            self.draw_menu_button(
                button_x,
                button_y,
                button_width,
                button_height,
                &label,
                index == selected_index,
            );
            button_y += 58;
        }

        self.draw_centered_text(
            self.height.saturating_sub(42),
            "W S O FLECHAS MOVER    ENTER ELEGIR    ESC VOLVER",
            0x7f8c95,
            2,
        );
    }

    /// Dibuja la pantalla de controles y objetivo inmediato.
    pub fn render_instructions_screen(&mut self) {
        self.draw_menu_background();
        self.draw_info_panel(
            "INSTRUCCIONES",
            &[
                "W O FLECHA ARRIBA AVANZAR",
                "S O FLECHA ABAJO RETROCEDER",
                "A D O FLECHAS MOVERSE LATERAL",
                "MOUSE GIRAR CAMARA",
                "Q EMITIR SONIDO",
                "BUSCA LA TARJETA ACTIVA EL TERMINAL",
                "ABRE LA PUERTA Y LLEGA A LA SALIDA",
            ],
        );
    }

    /// Dibuja la pantalla que explica el contexto del juego.
    pub fn render_about_screen(&mut self) {
        self.draw_menu_background();
        self.draw_info_panel(
            "DE QUE TRATA",
            &[
                "ESTAS ATRAPADO EN UN COMPLEJO ABANDONADO",
                "UNA PARED ANOMALA DESPIERTA SI TE ACERCAS",
                "SI TE ALCANZA VUELVES AL INICIO",
                "RUN FROM YE ES UNA CARRERA DE TENSION",
                "EXPLORA RECOGE LA TARJETA Y ESCAPA",
            ],
        );
    }

    /// Dibuja la pantalla de exito al completar un nivel.
    pub fn render_level_success_screen(&mut self, level_number: usize, has_next_level: bool) {
        self.draw_menu_background();

        let title = if has_next_level {
            format!("LEVEL {} SECURED", level_number)
        } else {
            "ALL LEVELS SECURED".to_string()
        };
        let subtitle = if has_next_level {
            "NEXT LEVEL READY"
        } else {
            "SITE FULLY CONTAINED"
        };
        let action = if has_next_level {
            "ENTER CONTINUE"
        } else {
            "ENTER MAIN MENU"
        };

        self.draw_centered_text(86, &title, 0xf5f3e7, 7);
        self.draw_centered_text(150, subtitle, 0xffc857, 4);

        let panel_width = 700.min(self.width.saturating_sub(48));
        let panel_height = 180.min(self.height.saturating_sub(220));
        let panel_x = self.width.saturating_sub(panel_width) / 2;
        let panel_y = 220;

        self.fill_rect(panel_x, panel_y, panel_width, panel_height, 0x4f6472);
        self.fill_rect(
            panel_x + 3,
            panel_y + 3,
            panel_width.saturating_sub(6),
            panel_height.saturating_sub(6),
            0x10161b,
        );
        self.fill_rect(
            panel_x + 14,
            panel_y + 14,
            8,
            panel_height.saturating_sub(28),
            0x52b788,
        );
        self.fill_rect(
            panel_x + panel_width.saturating_sub(22),
            panel_y + 14,
            8,
            panel_height.saturating_sub(28),
            0x52b788,
        );

        self.draw_centered_text(panel_y + 48, "ANOMALY ROUTE CLEARED", 0xbfd0d8, 3);
        self.draw_centered_text(panel_y + 90, action, 0xffffff, 4);
        self.draw_centered_text(self.height.saturating_sub(48), "ESC MAIN MENU", 0x9db4c4, 3);
    }

    /// Dibuja las paredes visibles usando raycasting columna por columna.
    fn draw_walls(&mut self, map: &Map, player: &Player) {
        let dir_x = player.angle.cos();
        let dir_y = player.angle.sin();
        // Plano de camara perpendicular a la direccion del jugador.
        let plane_x = -dir_y * FOV_FACTOR;
        let plane_y = dir_x * FOV_FACTOR;

        for screen_x in 0..self.width {
            // `camera_x` va de -1 a 1 y representa la columna dentro del FOV.
            let camera_x = 2.0 * screen_x as f32 / self.width as f32 - 1.0;
            let ray_dir_x = dir_x + plane_x * camera_x;
            let ray_dir_y = dir_y + plane_y * camera_x;
            let hit = cast_ray(map, player.x, player.y, ray_dir_x, ray_dir_y);
            self.depth_buffer[screen_x] = hit.distance;

            let line_height = (self.height as f32 / hit.distance.max(0.001)) as i32;
            let center_y = self.height as i32 / 2;
            let draw_start = (-line_height / 2 + center_y).max(0) as usize;
            let draw_end = (line_height / 2 + center_y).min(self.height as i32 - 1) as usize;
            // Avance vertical en la textura por cada pixel de pantalla.
            let texture_step = TEXTURE_SIZE as f32 / line_height.max(1) as f32;
            let mut texture_y =
                (draw_start as i32 - center_y + line_height / 2) as f32 * texture_step;
            let shade_factor = wall_shade_factor(hit.side, hit.distance);

            for y in draw_start..=draw_end {
                // Se muestrea la textura y luego se oscurece por distancia/lado.
                let color = shade_color(
                    wall_texel(hit.wall_id, hit.texture_x, texture_y as usize),
                    shade_factor,
                );
                self.buffer[y * self.width + screen_x] = color;
                texture_y += texture_step;
            }
        }
    }

    /// Dibuja objetos 2D dentro del mundo 3D.
    fn draw_sprites(&mut self, map: &Map, player: &Player, chaser: &Chaser) {
        let mut sprites = Vec::new();

        // Recolecta tiles especiales que se representan como sprites.
        for y in 0..map.height() {
            for x in 0..map.width() {
                let tile = map.tile_at(x as i32, y as i32);

                if matches!(tile, TILE_KEY | TILE_SWITCH | TILE_EXIT) {
                    let sprite_x = x as f32 + 0.5;
                    let sprite_y = y as f32 + 0.5;
                    let distance = (sprite_x - player.x).hypot(sprite_y - player.y);
                    sprites.push((distance, sprite_x, sprite_y, tile));
                }
            }
        }

        let chaser_tile = if chaser.disgusted() {
            SPRITE_CHASER_DISGUSTED
        } else if chaser.active() {
            SPRITE_CHASER_ACTIVE
        } else {
            SPRITE_CHASER_IDLE
        };
        let chaser_distance = (chaser.x - player.x).hypot(chaser.y - player.y);
        sprites.push((chaser_distance, chaser.x, chaser.y, chaser_tile));

        // Se dibuja de lejos a cerca para que los sprites se tapen bien entre si.
        sprites.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let dir_x = player.angle.cos();
        let dir_y = player.angle.sin();
        let plane_x = -dir_y * FOV_FACTOR;
        let plane_y = dir_x * FOV_FACTOR;
        let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);

        for (_, sprite_x, sprite_y, tile) in sprites {
            // Transforma la posicion del sprite desde mundo a espacio de camara.
            let rel_x = sprite_x - player.x;
            let rel_y = sprite_y - player.y;
            let transform_x = inv_det * (dir_y * rel_x - dir_x * rel_y);
            let transform_y = inv_det * (-plane_y * rel_x + plane_x * rel_y);

            if transform_y <= 0.05 {
                continue;
            }

            let screen_x = ((self.width as f32 / 2.0) * (1.0 + transform_x / transform_y)) as i32;
            let size_scale = if matches!(
                tile,
                SPRITE_CHASER_IDLE | SPRITE_CHASER_ACTIVE | SPRITE_CHASER_DISGUSTED
            ) {
                0.96
            } else {
                0.46
            };
            let sprite_size = (self.height as f32 * size_scale / transform_y.abs()) as i32;
            let center_y = self.height as i32 / 2;
            let start_y = (-sprite_size / 2 + center_y).max(0);
            let end_y = (sprite_size / 2 + center_y).min(self.height as i32 - 1);
            let start_x = (-sprite_size / 2 + screen_x).max(0);
            let end_x = (sprite_size / 2 + screen_x).min(self.width as i32 - 1);

            for stripe in start_x..=end_x {
                let stripe_index = stripe as usize;

                // Si una pared esta mas cerca en esta columna, el sprite queda oculto.
                if transform_y >= self.depth_buffer[stripe_index] {
                    continue;
                }

                for y in start_y..=end_y {
                    if let Some(color) = self.sample_sprite_color(
                        tile,
                        stripe - start_x,
                        y - start_y,
                        sprite_size.max(1),
                    ) {
                        self.buffer[y as usize * self.width + stripe_index] = color;
                    }
                }
            }
        }
    }

    /// Dibuja una vista superior compacta del mapa.
    fn draw_minimap(&mut self, map: &Map, player: &Player, chaser: &Chaser) {
        let max_width_scale =
            self.width.saturating_sub(MINIMAP_PADDING * 2).max(1) / map.width().max(1);
        let max_height_scale =
            self.height.saturating_sub(MINIMAP_PADDING * 2).max(1) / map.height().max(1);
        let scale = MINIMAP_CELL_SIZE
            .min(max_width_scale)
            .min(max_height_scale)
            .max(1);
        let map_width = map.width() * scale;
        let map_height = map.height() * scale;
        let origin_x = self.width.saturating_sub(map_width + MINIMAP_PADDING);
        let origin_y = MINIMAP_PADDING;
        let cell_size = scale.saturating_sub(1).max(1);

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
                    cell_size,
                    cell_size,
                    color,
                );
            }
        }

        let player_x = origin_x as i32 + (player.x * scale as f32) as i32;
        let player_y = origin_y as i32 + (player.y * scale as f32) as i32;
        let player_radius = (scale as i32 / 2).max(2);
        self.fill_circle(player_x, player_y, player_radius, 0xffffff);

        let direction_length = (scale as f32 * 2.0).max(6.0);
        let line_x = player_x + (player.angle.cos() * direction_length) as i32;
        let line_y = player_y + (player.angle.sin() * direction_length) as i32;
        self.draw_line(player_x, player_y, line_x, line_y, 0xfff36b);

        let chaser_x = origin_x as i32 + (chaser.x * scale as f32) as i32;
        let chaser_y = origin_y as i32 + (chaser.y * scale as f32) as i32;
        let chaser_color = if chaser.disgusted() {
            0xffc857
        } else if chaser.active() {
            0xe63946
        } else {
            0x9aa4ad
        };
        self.fill_rect(
            (chaser_x - player_radius).max(0) as usize,
            (chaser_y - player_radius).max(0) as usize,
            (player_radius * 2 + 1) as usize,
            (player_radius * 2 + 1) as usize,
            chaser_color,
        );

        let cooldown_ratio = player.sound_ability_cooldown_ratio();

        if cooldown_ratio > 0.0 {
            self.draw_sound_ability_cooldown_indicator(
                (origin_x + map_width / 2) as i32,
                (origin_y + map_height + 24) as i32,
                cooldown_ratio,
            );
        }
    }

    /// Dibuja textos de estado: FPS, vida, llave, puerta y objetivo.
    fn draw_hud(&mut self, map: &Map, player: &Player, chaser: &Chaser, fps: u32, message: &str) {
        self.draw_text(12, 12, &format!("FPS {}", fps), 0xffffff, 3);
        self.draw_text(12, 34, &format!("HP {}", player.lives), 0xfff0a3, 3);

        let key_text = if map.has_key() { "CARD YES" } else { "CARD NO" };
        let gate_text = if map.gate_open() {
            "DOOR OPEN"
        } else {
            "DOOR SHUT"
        };

        self.draw_text(12, 60, key_text, 0xffdd57, 2);
        self.draw_text(12, 78, gate_text, 0x9db4c4, 2);
        self.draw_text(
            12,
            96,
            if chaser.disgusted() {
                "ANOMALY REPULSED"
            } else if chaser.active() {
                "ANOMALY HUNTING"
            } else {
                "ANOMALY DORMANT"
            },
            if chaser.disgusted() {
                0xffc857
            } else if chaser.active() {
                0xff6b6b
            } else {
                0xa9b4c4
            },
            2,
        );
        self.draw_text(12, self.height.saturating_sub(34), message, 0xffffff, 3);
        self.draw_text(
            12,
            self.height.saturating_sub(54),
            "MOUSE TURN  A D STRAFE  Q SOUND  ESC EXIT",
            0xa9b4c4,
            2,
        );
    }

    /// Indicador circular del cooldown de la habilidad sonora.
    fn draw_sound_ability_cooldown_indicator(&mut self, center_x: i32, center_y: i32, ratio: f32) {
        self.fill_circle(center_x, center_y, 15, 0x101319);
        self.fill_clockwise_circle_slice(center_x, center_y, 12, ratio, 0xffc857);
        self.fill_circle(center_x, center_y, 7, 0x202631);
        self.fill_circle(center_x, center_y, 3, 0xffffff);
    }

    /// Devuelve el color de un sprite, usando textura externa para el chaser.
    fn sample_sprite_color(&self, tile: u8, local_x: i32, local_y: i32, size: i32) -> Option<u32> {
        if tile == SPRITE_CHASER_DISGUSTED {
            if let Some(texture) = &self.disgust_chaser_texture {
                return texture.sample(local_x, local_y, size);
            }

            if let Some(texture) = &self.chaser_texture {
                return texture.sample(local_x, local_y, size);
            }
        }

        if matches!(tile, SPRITE_CHASER_IDLE | SPRITE_CHASER_ACTIVE) {
            if let Some(texture) = &self.chaser_texture {
                return texture
                    .sample(local_x, local_y, size)
                    .map(|color| chaser_tint(color, tile == SPRITE_CHASER_ACTIVE));
            }
        }

        sprite_color(tile, local_x, local_y, size)
    }

    /// Rellena una porcion circular en sentido horario.
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

    /// Pinta un pixel si esta dentro del buffer.
    fn put_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            self.buffer[y * self.width + x] = color;
        }
    }

    /// Rellena un rectangulo recortandolo contra los bordes de pantalla.
    fn fill_rect(&mut self, x: usize, y: usize, rect_width: usize, rect_height: usize, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }

        let end_x = x.saturating_add(rect_width).min(self.width);
        let end_y = y.saturating_add(rect_height).min(self.height);

        for draw_y in y..end_y {
            let row_start = draw_y * self.width;
            self.buffer[row_start + x..row_start + end_x].fill(color);
        }
    }

    /// Rellena un circulo mediante prueba de distancia al centro.
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

    /// Dibuja una linea usando el algoritmo incremental de Bresenham.
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

    /// Dibuja texto usando glifos bitmap de 3x5 pixeles.
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

    /// Dibuja texto centrado horizontalmente.
    fn draw_centered_text(&mut self, y: usize, text: &str, color: u32, scale: usize) {
        let text_width = text_pixel_width(text, scale);
        let x = self.width.saturating_sub(text_width) / 2;
        self.draw_text(x, y, text, color, scale);
    }

    /// Fondo del menu con bandas y lineas de alerta discretas.
    fn draw_menu_background(&mut self) {
        for y in 0..self.height {
            let shade = if y < self.height / 2 {
                12 + y as u32 * 18 / self.height.max(1) as u32
            } else {
                24_u32.saturating_sub((y - self.height / 2) as u32 * 14 / self.height.max(1) as u32)
            };
            let color = rgb(shade + 4, shade + 7, shade + 5);
            let row_start = y * self.width;
            self.buffer[row_start..row_start + self.width].fill(color);
        }

        for x in (0..self.width).step_by(48) {
            self.fill_rect(x, 0, 1, self.height, 0x172027);
        }

        for y in (0..self.height).step_by(36) {
            self.fill_rect(0, y, self.width, 1, 0x141c22);
        }

        self.fill_rect(0, 0, self.width, 8, 0x6e1c1c);
        self.fill_rect(0, self.height.saturating_sub(8), self.width, 8, 0x6e1c1c);
    }

    /// Boton del menu con borde de seleccion.
    fn draw_menu_button(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        label: &str,
        selected: bool,
    ) {
        let border_color = if selected { 0xffc857 } else { 0x4f6472 };
        let fill_color = if selected { 0x26313a } else { 0x141b21 };
        let text_color = if selected { 0xffffff } else { 0xb3c0c8 };

        self.fill_rect(x, y, width, height, border_color);
        self.fill_rect(
            x + 3,
            y + 3,
            width.saturating_sub(6),
            height.saturating_sub(6),
            fill_color,
        );

        if selected {
            self.fill_rect(x + 8, y + 8, 6, height.saturating_sub(16), 0xff5a1f);
            self.fill_rect(
                x + width.saturating_sub(14),
                y + 8,
                6,
                height.saturating_sub(16),
                0xff5a1f,
            );
        }

        let scale = 3;
        let text_x = x + width.saturating_sub(text_pixel_width(label, scale)) / 2;
        let text_y = y + height.saturating_sub(5 * scale) / 2;
        self.draw_text(text_x, text_y, label, text_color, scale);
    }

    /// Panel reutilizable para pantallas informativas del menu.
    fn draw_info_panel(&mut self, title: &str, lines: &[&str]) {
        self.draw_centered_text(78, title, 0xf5f3e7, 6);

        let panel_width = 760.min(self.width.saturating_sub(48));
        let panel_height = 300.min(self.height.saturating_sub(180));
        let panel_x = self.width.saturating_sub(panel_width) / 2;
        let panel_y = 170;

        self.fill_rect(panel_x, panel_y, panel_width, panel_height, 0x4f6472);
        self.fill_rect(
            panel_x + 3,
            panel_y + 3,
            panel_width.saturating_sub(6),
            panel_height.saturating_sub(6),
            0x10161b,
        );

        let mut y = panel_y + 32;

        for line in lines {
            self.draw_centered_text(y, line, 0xbfd0d8, 3);
            y += 32;
        }

        self.draw_centered_text(
            self.height.saturating_sub(48),
            "ENTER O ESPACIO VOLVER",
            0xffc857,
            3,
        );
    }

    /// Dibuja un caracter individual de la fuente bitmap.
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

impl SpriteTexture {
    fn sample(&self, local_x: i32, local_y: i32, size: i32) -> Option<u32> {
        if local_x < 0 || local_y < 0 || local_x >= size || local_y >= size {
            return None;
        }

        let texture_x = (local_x as usize * self.width / size.max(1) as usize).min(self.width - 1);
        let texture_y =
            (local_y as usize * self.height / size.max(1) as usize).min(self.height - 1);
        let color = self.pixels[texture_y * self.width + texture_x];

        if is_chaser_texture_transparent(color) {
            None
        } else {
            Some(color)
        }
    }
}

fn load_chaser_texture(label: &str, asset_names: &[&str]) -> Option<SpriteTexture> {
    for path in chaser_texture_paths(asset_names) {
        if let Some(texture) = load_sprite_texture(&path) {
            eprintln!("Loaded {label} texture: {}", path.display());
            return Some(texture);
        }
    }

    eprintln!("{label} texture not found. Using procedural fallback.");
    None
}

fn chaser_texture_paths(asset_names: &[&str]) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"));

    if let Ok(current_dir) = std::env::current_dir() {
        roots.push(current_dir.join("assets"));
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            roots.push(exe_dir.join("assets"));
        }
    }

    roots
        .into_iter()
        .flat_map(|root| {
            asset_names
                .iter()
                .map(move |asset_name| root.join(asset_name))
        })
        .collect()
}

fn load_sprite_texture(path: &Path) -> Option<SpriteTexture> {
    let image = image::ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?
        .to_rgba8();
    let (width, height) = image.dimensions();
    let pixels = image
        .pixels()
        .map(|pixel| {
            let [red, green, blue, alpha] = pixel.0;

            if alpha < 16 {
                0xff00ff
            } else {
                rgb(red as u32, green as u32, blue as u32)
            }
        })
        .collect();

    Some(SpriteTexture {
        width: width as usize,
        height: height as usize,
        pixels,
    })
}

fn chaser_tint(color: u32, active: bool) -> u32 {
    if active {
        let red = (((color >> 16) & 0xff) + 42).min(255);
        let green = (((color >> 8) & 0xff) * 72 / 100).min(255);
        let blue = ((color & 0xff) * 72 / 100).min(255);

        rgb(red, green, blue)
    } else {
        shade_color(color, 0.86)
    }
}

fn is_chaser_texture_transparent(color: u32) -> bool {
    color == 0xff00ff
}

/// Lanza un rayo desde el jugador y devuelve el primer tile bloqueante.
fn cast_ray(map: &Map, pos_x: f32, pos_y: f32, ray_dir_x: f32, ray_dir_y: f32) -> RayHit {
    let mut map_x = pos_x.floor() as i32;
    let mut map_y = pos_y.floor() as i32;
    // Distancia que el rayo avanza para cruzar una celda en X o en Y.
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

    // Direccion de avance en la grilla y distancia inicial al primer borde.
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

    // DDA: avanza de celda en celda hasta encontrar algo que bloquee vision.
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

        if matches!(wall_id, TILE_WALL | TILE_GATE | TILE_METAL | TILE_RUINS) {
            break;
        }
    }

    // Distancia perpendicular para evitar efecto ojo de pez.
    let distance = if side == 0 {
        (map_x as f32 - pos_x + (1 - step_x) as f32 / 2.0) / ray_dir_x
    } else {
        (map_y as f32 - pos_y + (1 - step_y) as f32 / 2.0) / ray_dir_y
    }
    .abs();

    // Coordenada exacta del impacto dentro de la pared, usada para tex_x.
    let wall_x = if side == 0 {
        pos_y + distance * ray_dir_y
    } else {
        pos_x + distance * ray_dir_x
    };
    let wall_x = wall_x - wall_x.floor();
    let mut texture_x = ((wall_x * TEXTURE_SIZE as f32) as usize).min(TEXTURE_SIZE - 1);

    // Invierte la textura segun la cara golpeada para mantener orientacion coherente.
    if (side == 0 && ray_dir_x > 0.0) || (side == 1 && ray_dir_y < 0.0) {
        texture_x = TEXTURE_SIZE.saturating_sub(texture_x + 1);
    }

    RayHit {
        distance,
        wall_id,
        side,
        texture_x,
    }
}

/// Color usado por cada tile en el minimapa.
fn minimap_color(tile: u8) -> u32 {
    match tile {
        TILE_WALL => 0x4a5a52,
        TILE_GATE => 0xffb703,
        TILE_METAL => 0x7c92a3,
        TILE_RUINS => 0x39b24a,
        TILE_KEY => 0xffdd57,
        TILE_SWITCH => 0x4cc9f0,
        TILE_EXIT => 0xb7efc5,
        _ => 0xc9d1c8,
    }
}

/// Devuelve el color de un sprite procedural o `None` para pixeles transparentes.
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
        SPRITE_CHASER_IDLE | SPRITE_CHASER_ACTIVE | SPRITE_CHASER_DISGUSTED => {
            let inside = nx.abs() < 0.72 && ny.abs() < 0.88;
            let border = nx.abs() > 0.62 || ny.abs() > 0.78;
            let mortar = local_y.rem_euclid((size / 5).max(4)) <= 1
                || (local_x + (local_y / 12) * 7).rem_euclid((size / 4).max(5)) <= 1;
            let alert_crack =
                tile == SPRITE_CHASER_ACTIVE && (local_x * 3 + local_y * 5).rem_euclid(31) < 3;
            let disgust_crack =
                tile == SPRITE_CHASER_DISGUSTED && (local_x * 7 + local_y * 2).rem_euclid(29) < 4;

            if !inside {
                None
            } else if alert_crack {
                Some(0xff5a1f)
            } else if disgust_crack {
                Some(0xffc857)
            } else if border {
                Some(if tile == SPRITE_CHASER_ACTIVE {
                    0x4a1010
                } else if tile == SPRITE_CHASER_DISGUSTED {
                    0x4a3520
                } else {
                    0x2a2d2f
                })
            } else if mortar {
                Some(if tile == SPRITE_CHASER_ACTIVE {
                    0x9c2a2a
                } else if tile == SPRITE_CHASER_DISGUSTED {
                    0xa16a36
                } else {
                    0x3a3d3f
                })
            } else {
                Some(if tile == SPRITE_CHASER_ACTIVE {
                    0x6e1c1c
                } else if tile == SPRITE_CHASER_DISGUSTED {
                    0x7b4a24
                } else {
                    0x4a4f52
                })
            }
        }
        _ => None,
    }
}

/// Precalcula techo y piso para copiarlo al inicio de cada frame.
///
/// Tonos bajos y con tinte verdoso, como luz de emergencia en un complejo
/// que quedo funcionando solo con energia de respaldo.
fn build_background(width: usize, height: usize) -> Vec<u32> {
    let mut background = vec![0; width * height];
    let horizon = height / 2;

    for y in 0..horizon {
        let shade = 10 + (y as u32 * 16 / horizon as u32);
        let color = rgb(shade, shade + 5, shade + 2);
        let row_start = y * width;
        background[row_start..row_start + width].fill(color);
    }

    for y in horizon..height {
        let depth = (y - horizon) as u32;
        let shade = 26_u32.saturating_sub(depth * 14 / horizon as u32);
        let color = rgb(shade + 6, shade + 8, shade + 6);
        let row_start = y * width;
        background[row_start..row_start + width].fill(color);
    }

    background
}

/// Factor de sombreado por lado y distancia para dar profundidad.
///
/// El minimo de oscuridad se bajo bastante: lo que esta lejos casi
/// desaparece, para que el jugador dependa de acercarse (o del minimapa)
/// en vez de ver todo el pasillo de un vistazo.
fn wall_shade_factor(side: i32, distance: f32) -> f32 {
    let side_factor = if side == 1 { 0.72 } else { 1.0 };
    let distance_factor = (1.0 / (1.0 + distance * 0.11)).clamp(0.16, 1.0);

    side_factor * distance_factor
}

/// Aplica un factor de sombreado ya calculado a un color RGB.
fn shade_color(color: u32, factor: f32) -> u32 {
    let r = (((color >> 16) & 0xff) as f32 * factor) as u32;
    let g = (((color >> 8) & 0xff) as f32 * factor) as u32;
    let b = ((color & 0xff) as f32 * factor) as u32;

    rgb(r, g, b)
}

/// Empaca componentes RGB en el formato `0xRRGGBB`.
fn rgb(r: u32, g: u32, b: u32) -> u32 {
    (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
}

/// Calcula ancho aproximado de una cadena con la fuente bitmap.
fn text_pixel_width(text: &str, scale: usize) -> usize {
    text.chars().count() * 4 * scale
}

/// Patron bitmap de una letra o numero para el HUD.
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
