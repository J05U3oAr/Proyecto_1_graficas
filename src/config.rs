//! Constantes de configuracion global.
//!
//! Mantener estos valores aqui permite ajustar la ventana, movimiento,
//! raycasting y HUD sin buscar numeros magicos por todo el codigo.

/// Ancho de la ventana y del buffer de pixeles.
pub const SCREEN_WIDTH: usize = 960;
/// Alto de la ventana y del buffer de pixeles.
pub const SCREEN_HEIGHT: usize = 540;
/// FPS objetivo que `minifb` intenta respetar.
pub const TARGET_FPS: usize = 60;

/// Velocidad base de movimiento del jugador en celdas por segundo.
pub const MOVE_SPEED: f32 = 3.15;
/// Velocidad de giro del jugador en radianes por segundo.
pub const ROTATION_SPEED: f32 = 2.4;
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

/// Tamano en pixeles de cada celda del minimapa.
pub const MINIMAP_CELL_SIZE: usize = 10;
/// Separacion del minimapa respecto al borde de la ventana.
pub const MINIMAP_PADDING: usize = 12;
