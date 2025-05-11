mod attribute;
mod bullet;
mod enemy;
mod gun;
mod player;

use godot::prelude::*;

mod enemy_drop;
mod room;
mod utils;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
