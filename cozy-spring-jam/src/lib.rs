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

use godot::classes::{CharacterBody2D, ICharacterBody2D};

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self { speed: 300.0, base }
    }

    fn physics_process(&mut self, _delta: f64) {
        let input = Input::singleton();

        let left = input.is_action_pressed("move_left");
        let right = input.is_action_pressed("move_right");
        let up = input.is_action_pressed("move_up");
        let down = input.is_action_pressed("move_down");

        let mut movement_vec = Vector2::new(0.0, 0.0);
        if left {
            movement_vec.x -= 1.0;
        }
        if right {
            movement_vec.x += 1.0;
        }
        if up {
            movement_vec.y -= 1.0;
        }
        if down {
            movement_vec.y += 1.0;
        }
        let velocity = movement_vec.normalized_or_zero() * self.speed;

        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();
    }
}
