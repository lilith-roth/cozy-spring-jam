mod bullet;
mod enemy;
mod player;

use godot::prelude::*;

mod room;
mod utils;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
