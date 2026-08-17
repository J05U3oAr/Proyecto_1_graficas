//! Punto de entrada del juego.
//!
//! Este archivo solo conecta los modulos y arranca el loop principal.

mod config;
mod game;
mod input;
mod map;
mod player;
mod renderer;
mod texture;

use game::Game;

fn main() -> Result<(), minifb::Error> {
    // Crea el estado inicial del juego y mantiene la ventana activa.
    let mut game = Game::new()?;
    game.run()
}
