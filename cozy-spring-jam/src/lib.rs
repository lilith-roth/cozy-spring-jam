mod player;
mod enemy;

use godot::classes::CharacterBody2D;
use godot::prelude::*;

mod room;
mod utils;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

