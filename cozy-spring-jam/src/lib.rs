mod enemy;
mod player;

use godot::prelude::*;

mod bullet;
mod room;
mod utils;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
