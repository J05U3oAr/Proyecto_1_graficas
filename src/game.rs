//! Orquestador principal del juego.
//!
//! Aqui vive el loop: leer input, actualizar mundo, renderizar y enviar
//! el buffer a la ventana.

use std::time::{Duration, Instant};

use minifb::{Key, KeyRepeat, Scale, ScaleMode, Window, WindowOptions};

use crate::{
    audio::play_disgust_sound,
    chaser::{Chaser, ChaserEvent},
    config::{BORDERLESS_FULLSCREEN, MESSAGE_DURATION, SCREEN_HEIGHT, SCREEN_WIDTH, TARGET_FPS},
    input::InputState,
    map::Map,
    player::Player,
    renderer::Renderer,
};

/// Estado global de una partida.
pub struct Game {
    /// Ventana creada con `minifb`.
    window: Window,
    /// Renderer que dibuja todos los pixeles del frame.
    renderer: Renderer,
    /// Mapa actual y estado de sus interacciones.
    map: Map,
    /// Jugador, incluyendo posicion, angulo, vida y habilidad sonora.
    player: Player,
    /// Pared que se activa por rango y persigue al jugador.
    chaser: Chaser,
    /// Tiempo del frame anterior para calcular delta time.
    previous_frame: Instant,
    /// Reloj usado para refrescar el contador visible de FPS.
    fps_timer: Instant,
    /// Frames acumulados durante el segundo actual.
    frame_counter: u32,
    /// Ultimo valor de FPS mostrado.
    displayed_fps: u32,
    /// Mensaje actual del HUD.
    message: &'static str,
    /// Tiempo restante para mostrar un mensaje temporal.
    message_timer: f32,
    /// Indica si ya se centro el mouse al menos una vez.
    mouse_centered: bool,
    /// Pantalla activa del juego.
    screen: GameScreen,
    /// Opcion seleccionada en el menu principal.
    menu_selection: usize,
}

impl Game {
    /// Crea la ventana, el mapa, el renderer y al jugador en su spawn.
    pub fn new() -> Result<Self, minifb::Error> {
        let (window_width, window_height) = window_size();
        let mut window = Window::new(
            "RUN FROM YE",
            window_width,
            window_height,
            WindowOptions {
                borderless: BORDERLESS_FULLSCREEN,
                title: !BORDERLESS_FULLSCREEN,
                resize: false,
                scale: Scale::X1,
                scale_mode: ScaleMode::Stretch,
                topmost: BORDERLESS_FULLSCREEN,
                ..WindowOptions::default()
            },
        )?;

        window.set_target_fps(TARGET_FPS);
        window.set_cursor_visibility(false);

        if BORDERLESS_FULLSCREEN {
            window.set_position(0, 0);
            center_mouse(&window);
        }

        let map = Map::level_one();
        let (player_x, player_y, player_angle) = map.player_spawn();
        let (chaser_x, chaser_y) = map.chaser_spawn();

        Ok(Self {
            window,
            renderer: Renderer::new(SCREEN_WIDTH, SCREEN_HEIGHT),
            map,
            player: Player::new(player_x, player_y, player_angle),
            chaser: Chaser::new(chaser_x, chaser_y),
            previous_frame: Instant::now(),
            fps_timer: Instant::now(),
            frame_counter: 0,
            displayed_fps: 0,
            message: "FIND ACCESS CARD",
            message_timer: MESSAGE_DURATION,
            mouse_centered: false,
            screen: GameScreen::MainMenu,
            menu_selection: 0,
        })
    }

    /// Ejecuta el loop principal hasta cerrar la ventana o salir desde el menu.
    pub fn run(&mut self) -> Result<(), minifb::Error> {
        while self.window.is_open() {
            let now = Instant::now();
            let dt = (now - self.previous_frame).as_secs_f32();
            self.previous_frame = now;

            if !self.update_screen(dt, now) {
                break;
            }

            self.window
                .update_with_buffer(self.renderer.buffer(), SCREEN_WIDTH, SCREEN_HEIGHT)?;
        }

        Ok(())
    }

    /// Actualiza la pantalla activa y dibuja el frame correspondiente.
    fn update_screen(&mut self, dt: f32, now: Instant) -> bool {
        match self.screen {
            GameScreen::MainMenu => self.update_main_menu(),
            GameScreen::Instructions => self.update_instructions(),
            GameScreen::About => self.update_about(),
            GameScreen::Playing => self.update_playing(dt, now),
        }
    }

    /// Lee navegacion del menu principal y dibuja sus opciones.
    fn update_main_menu(&mut self) -> bool {
        const MENU_OPTIONS: usize = 4;

        self.window.set_title("RUN FROM YE");

        if self.window.is_key_pressed(Key::Escape, KeyRepeat::No) {
            return false;
        }

        if self.window.is_key_pressed(Key::Up, KeyRepeat::No)
            || self.window.is_key_pressed(Key::W, KeyRepeat::No)
        {
            self.menu_selection = if self.menu_selection == 0 {
                MENU_OPTIONS - 1
            } else {
                self.menu_selection - 1
            };
        }

        if self.window.is_key_pressed(Key::Down, KeyRepeat::No)
            || self.window.is_key_pressed(Key::S, KeyRepeat::No)
        {
            self.menu_selection = (self.menu_selection + 1) % MENU_OPTIONS;
        }

        if self.window.is_key_pressed(Key::Enter, KeyRepeat::No)
            || self.window.is_key_pressed(Key::Space, KeyRepeat::No)
        {
            match self.menu_selection {
                0 => self.start_new_game(),
                1 => self.screen = GameScreen::Instructions,
                2 => self.screen = GameScreen::About,
                3 => return false,
                _ => {}
            }
        }

        self.renderer.render_main_menu(self.menu_selection);
        true
    }

    /// Dibuja la pantalla de instrucciones hasta que el jugador vuelva.
    fn update_instructions(&mut self) -> bool {
        if self.window.is_key_pressed(Key::Escape, KeyRepeat::No)
            || self.window.is_key_pressed(Key::Enter, KeyRepeat::No)
            || self.window.is_key_pressed(Key::Space, KeyRepeat::No)
        {
            self.screen = GameScreen::MainMenu;
        }

        self.renderer.render_instructions_screen();
        true
    }

    /// Dibuja la pantalla de contexto narrativo hasta que el jugador vuelva.
    fn update_about(&mut self) -> bool {
        if self.window.is_key_pressed(Key::Escape, KeyRepeat::No)
            || self.window.is_key_pressed(Key::Enter, KeyRepeat::No)
            || self.window.is_key_pressed(Key::Space, KeyRepeat::No)
        {
            self.screen = GameScreen::MainMenu;
        }

        self.renderer.render_about_screen();
        true
    }

    /// Actualiza una partida en curso.
    fn update_playing(&mut self, dt: f32, now: Instant) -> bool {
        if self.window.is_key_pressed(Key::Escape, KeyRepeat::No) {
            self.screen = GameScreen::MainMenu;
            self.mouse_centered = false;
            return true;
        }

        let mouse_delta_x = self.read_mouse_delta_x();
        let input = InputState::from_window(&self.window, mouse_delta_x);
        let sound_used = self.player.update(&input, &self.map, dt);

        if sound_used {
            self.activate_disgust_sound();
        }

        self.update_chaser(dt);
        self.update_interactions(dt);
        self.update_fps(now);

        self.renderer.render(
            &self.map,
            &self.player,
            &self.chaser,
            self.displayed_fps,
            self.message,
        );

        true
    }

    /// Reinicia el estado jugable y entra al nivel.
    fn start_new_game(&mut self) {
        self.map = Map::level_one();
        let (player_x, player_y, player_angle) = self.map.player_spawn();
        let (chaser_x, chaser_y) = self.map.chaser_spawn();

        self.player = Player::new(player_x, player_y, player_angle);
        self.chaser = Chaser::new(chaser_x, chaser_y);
        self.message = "FIND ACCESS CARD";
        self.message_timer = MESSAGE_DURATION;
        self.previous_frame = Instant::now();
        self.mouse_centered = false;
        self.screen = GameScreen::Playing;
    }

    /// Calcula cuanto se movio el cursor horizontalmente desde el frame anterior.
    fn read_mouse_delta_x(&mut self) -> f32 {
        if !self.mouse_centered {
            center_mouse(&self.window);
            self.mouse_centered = true;
            return 0.0;
        }

        let delta_x = mouse_delta_x_from_center(&self.window);
        center_mouse(&self.window);

        if delta_x.abs() < 0.75 {
            0.0
        } else {
            // Evita saltos bruscos si el sistema reporta un movimiento muy grande.
            delta_x.clamp(-80.0, 80.0)
        }
    }

    /// Actualiza el contador de FPS una vez por segundo.
    fn update_fps(&mut self, now: Instant) {
        self.frame_counter += 1;

        if now.duration_since(self.fps_timer) >= Duration::from_secs(1) {
            self.displayed_fps = self.frame_counter;
            self.frame_counter = 0;
            self.fps_timer = now;

            self.window
                .set_title(&format!("RUN FROM YE | FPS: {}", self.displayed_fps));
        }
    }

    /// Actualiza persecucion, dano por contacto y mensajes de la pared movil.
    fn update_chaser(&mut self, dt: f32) {
        if let Some(event) = self.chaser.update(&self.map, &self.player, dt) {
            self.message = match event {
                ChaserEvent::Spotted => "ANOMALY DETECTED",
                ChaserEvent::Lost => "ANOMALY LOST",
                ChaserEvent::HitPlayer => {
                    self.player.take_hit_and_respawn();
                    self.chaser.reset();
                    "CONTACT REINITIALIZING"
                }
            };
            self.message_timer = MESSAGE_DURATION;
        }
    }

    /// Reproduce el sonido de la habilidad y repele a la pared si esta cerca.
    fn activate_disgust_sound(&mut self) {
        play_disgust_sound();

        self.message = if self.chaser.disgust(&self.player) {
            "ANOMALY DISGUSTED"
        } else {
            "SOUND ECHOES"
        };
        self.message_timer = MESSAGE_DURATION;
    }

    /// Procesa pickups, switch, gate y mensajes de objetivo.
    fn update_interactions(&mut self, dt: f32) {
        self.message_timer = (self.message_timer - dt).max(0.0);

        if let Some(message) = self.map.update_player_interactions(&mut self.player) {
            self.message = message;
            self.message_timer = MESSAGE_DURATION;
        } else if self.message_timer <= 0.0 {
            self.message = self.current_goal_message();
        }
    }

    /// Mensaje persistente que guia al jugador cuando no hay evento reciente.
    fn current_goal_message(&self) -> &'static str {
        if self.map.completed() {
            "SITE SECURED"
        } else if !self.map.has_key() {
            "FIND ACCESS CARD"
        } else if !self.map.switch_pressed() {
            "ACTIVATE TERMINAL"
        } else {
            "REACH EXIT"
        }
    }
}

/// Pantallas de alto nivel controladas por el loop principal.
#[derive(Clone, Copy)]
enum GameScreen {
    MainMenu,
    Instructions,
    About,
    Playing,
}

/// Tamano real de la ventana. En fullscreen se usa la resolucion del monitor.
fn window_size() -> (usize, usize) {
    if BORDERLESS_FULLSCREEN {
        primary_screen_size()
    } else {
        (SCREEN_WIDTH, SCREEN_HEIGHT)
    }
}

/// Resolucion del monitor principal en Windows.
#[cfg(target_os = "windows")]
fn primary_screen_size() -> (usize, usize) {
    use winapi::um::winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

    unsafe {
        (
            GetSystemMetrics(SM_CXSCREEN).max(SCREEN_WIDTH as i32) as usize,
            GetSystemMetrics(SM_CYSCREEN).max(SCREEN_HEIGHT as i32) as usize,
        )
    }
}

/// Fallback para plataformas donde no se consulta el monitor.
#[cfg(not(target_os = "windows"))]
fn primary_screen_size() -> (usize, usize) {
    (SCREEN_WIDTH, SCREEN_HEIGHT)
}

/// Recentra el cursor en la ventana para obtener movimiento relativo continuo.
#[cfg(target_os = "windows")]
fn center_mouse(window: &Window) {
    use winapi::um::winuser::SetCursorPos;

    let (center_x, center_y) = window_center(window);

    unsafe {
        SetCursorPos(center_x, center_y);
    }
}

/// Diferencia horizontal entre el cursor actual y el centro real de la ventana.
#[cfg(target_os = "windows")]
fn mouse_delta_x_from_center(window: &Window) -> f32 {
    use winapi::{shared::windef::POINT, um::winuser::GetCursorPos};

    let (center_x, _) = window_center(window);
    let mut cursor = POINT { x: 0, y: 0 };

    unsafe {
        if GetCursorPos(&mut cursor) == 0 {
            0.0
        } else {
            (cursor.x - center_x) as f32
        }
    }
}

/// Centro de la ventana en coordenadas absolutas de pantalla.
#[cfg(target_os = "windows")]
fn window_center(window: &Window) -> (i32, i32) {
    let (window_x, window_y) = window.get_position();
    let (width, height) = window.get_size();
    let center_x = window_x + width as isize / 2;
    let center_y = window_y + height as isize / 2;

    (center_x as i32, center_y as i32)
}

/// En otros sistemas el cursor no se recentra automaticamente.
#[cfg(not(target_os = "windows"))]
fn center_mouse(_window: &Window) {}

/// Fallback de mouse relativo para plataformas sin recenter automatico.
#[cfg(not(target_os = "windows"))]
fn mouse_delta_x_from_center(window: &Window) -> f32 {
    let center_x = window.get_size().0 as f32 / 2.0;

    window
        .get_mouse_pos(minifb::MouseMode::Pass)
        .map_or(0.0, |(mouse_x, _)| mouse_x - center_x)
}
