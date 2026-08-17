mod config;
mod game;
mod input;
mod map;
mod player;
mod renderer;
mod texture;

use game::Game;

fn main() -> Result<(), minifb::Error> {
    let mut game = Game::new()?;
    game.run()
}
