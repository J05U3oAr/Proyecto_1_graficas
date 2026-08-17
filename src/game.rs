//! Orquestador principal del juego.
//!
//! Aqui vive el loop: leer input, actualizar mundo, renderizar y enviar
//! el buffer a la ventana.

use std::time::{Duration, Instant};

use minifb::{Key, MouseMode, Scale, Window, WindowOptions};

use crate::{
    config::{MESSAGE_DURATION, SCREEN_HEIGHT, SCREEN_WIDTH, TARGET_FPS},
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
    /// Jugador, incluyendo posicion, angulo, vida y dash.
    player: Player,
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
    /// Ultima posicion horizontal conocida del mouse dentro de la ventana.
    previous_mouse_x: Option<f32>,
}

impl Game {
    /// Crea la ventana, el mapa, el renderer y al jugador en su spawn.
    pub fn new() -> Result<Self, minifb::Error> {
        let mut window = Window::new(
            "Ray Caster - Proyecto 1",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            WindowOptions {
                resize: false,
                scale: Scale::X1,
                ..WindowOptions::default()
            },
        )?;

        window.set_target_fps(TARGET_FPS);

        let map = Map::level_one();
        let (player_x, player_y, player_angle) = map.player_spawn();

        Ok(Self {
            window,
            renderer: Renderer::new(SCREEN_WIDTH, SCREEN_HEIGHT),
            map,
            player: Player::new(player_x, player_y, player_angle),
            previous_frame: Instant::now(),
            fps_timer: Instant::now(),
            frame_counter: 0,
            displayed_fps: 0,
            message: "FIND KEY",
            message_timer: MESSAGE_DURATION,
            previous_mouse_x: None,
        })
    }

    /// Ejecuta el loop principal hasta cerrar la ventana o presionar Escape.
    pub fn run(&mut self) -> Result<(), minifb::Error> {
        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            let now = Instant::now();
            let dt = (now - self.previous_frame).as_secs_f32();
            self.previous_frame = now;

            let mouse_delta_x = self.read_mouse_delta_x();
            let input = InputState::from_window(&self.window, mouse_delta_x);
            self.player.update(&input, &self.map, dt);
            self.update_interactions(dt);
            self.update_fps(now);

            self.renderer
                .render(&self.map, &self.player, self.displayed_fps, self.message);

            self.window
                .update_with_buffer(self.renderer.buffer(), SCREEN_WIDTH, SCREEN_HEIGHT)?;
        }

        Ok(())
    }

    /// Calcula cuanto se movio el cursor horizontalmente desde el frame anterior.
    fn read_mouse_delta_x(&mut self) -> f32 {
        let Some((mouse_x, _)) = self.window.get_mouse_pos(MouseMode::Discard) else {
            self.previous_mouse_x = None;
            return 0.0;
        };

        let delta_x = self
            .previous_mouse_x
            .map_or(0.0, |previous_x| mouse_x - previous_x);
        self.previous_mouse_x = Some(mouse_x);

        // Evita saltos bruscos cuando el cursor entra de nuevo a la ventana.
        delta_x.clamp(-80.0, 80.0)
    }

    /// Actualiza el contador de FPS una vez por segundo.
    fn update_fps(&mut self, now: Instant) {
        self.frame_counter += 1;

        if now.duration_since(self.fps_timer) >= Duration::from_secs(1) {
            self.displayed_fps = self.frame_counter;
            self.frame_counter = 0;
            self.fps_timer = now;

            self.window.set_title(&format!(
                "Ray Caster - Proyecto 1 | FPS: {}",
                self.displayed_fps
            ));
        }
    }

    /// Procesa pickups, hazards, switch, gate y mensajes de objetivo.
    fn update_interactions(&mut self, dt: f32) {
        self.message_timer = (self.message_timer - dt).max(0.0);

        if self.player.touched_hazard() {
            self.player.take_hit_and_respawn();
            self.message = "SPIKES HIT";
            self.message_timer = MESSAGE_DURATION;
        } else if let Some(message) = self.map.update_player_interactions(&mut self.player) {
            self.message = message;
            self.message_timer = MESSAGE_DURATION;
        } else if self.message_timer <= 0.0 {
            self.message = self.current_goal_message();
        }
    }

    /// Mensaje persistente que guia al jugador cuando no hay evento reciente.
    fn current_goal_message(&self) -> &'static str {
        if self.map.completed() {
            "LEVEL COMPLETE"
        } else if !self.map.has_key() {
            "FIND KEY"
        } else if !self.map.switch_pressed() {
            "PRESS SWITCH"
        } else {
            "REACH EXIT"
        }
    }
}
