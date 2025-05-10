mod enemy;
mod player;

use godot::prelude::*;

mod room;
mod utils;
mod bullet;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
