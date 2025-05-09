mod player;
mod enemy;

use godot::prelude::*;

mod room;
mod utils;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
struct Player {
    #[export]
    speed: f32,
    base: Base<CharacterBody2D>,
}


struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

