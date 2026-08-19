//! Constantes de configuracion global.
//!
//! Mantener estos valores aqui permite ajustar la ventana, movimiento,
//! raycasting y HUD sin buscar numeros magicos por todo el codigo.

/// Ancho del buffer interno de render.
pub const SCREEN_WIDTH: usize = 960;
/// Alto del buffer interno de render.
pub const SCREEN_HEIGHT: usize = 540;
/// Si esta activo, la ventana usa modo borderless del tamano de la pantalla.
pub const BORDERLESS_FULLSCREEN: bool = true;
/// FPS objetivo que `minifb` intenta respetar.
pub const TARGET_FPS: usize = 60;

/// Velocidad base de movimiento del jugador en celdas por segundo.
pub const MOVE_SPEED: f32 = 3.15;
/// Sensibilidad del mouse en radianes por pixel horizontal.
pub const MOUSE_SENSITIVITY: f32 = 0.0032;
/// Radio usado para calcular colisiones alrededor del jugador.
pub const PLAYER_RADIUS: f32 = 0.22;
/// Tamano del plano de camara; controla el campo de vision.
pub const FOV_FACTOR: f32 = 0.66;

/// Distancia que avanza el jugador al usar dash.
pub const DASH_DISTANCE: f32 = 1.35;
/// Tiempo que debe pasar antes de volver a usar dash.
pub const DASH_COOLDOWN: f32 = 2.4;
/// Tamano maximo de cada sub-paso al mover con colisiones.
pub const COLLISION_STEP: f32 = 0.07;
/// Duracion de los mensajes temporales del HUD.
pub const MESSAGE_DURATION: f32 = 2.8;

/// Velocidad del perseguidor en celdas por segundo.
pub const CHASER_SPEED: f32 = 2.2;
/// Distancia a la que la pared perseguidora detecta al jugador.
pub const CHASER_WAKE_DISTANCE: f32 = 7.0;
/// Distancia a la que el jugador logra perder a la pared perseguidora.
pub const CHASER_LOSE_DISTANCE: f32 = 11.0;
/// Radio de colision usado por la pared perseguidora.
pub const CHASER_RADIUS: f32 = 0.22;
/// Distancia a la que la pared perseguidora golpea al jugador.
pub const CHASER_HIT_DISTANCE: f32 = 0.56;
/// Frecuencia con la que recalcula su ruta hacia el jugador.
pub const CHASER_REPATH_INTERVAL: f32 = 0.18;

/// Tamano en pixeles de cada celda del minimapa.
pub const MINIMAP_CELL_SIZE: usize = 6;
/// Separacion del minimapa respecto al borde de la ventana.
pub const MINIMAP_PADDING: usize = 12;
