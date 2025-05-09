mod player;
mod enemy;

use godot::prelude::*;

mod room;
mod utils;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

