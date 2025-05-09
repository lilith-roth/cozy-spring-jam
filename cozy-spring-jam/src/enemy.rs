use godot::classes::{IRigidBody2D, RigidBody2D};
use godot::obj::Base;
use godot::prelude::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(base=RigidBody2D)]
struct Enemy {
    #[export]
    health: u16,
    #[export]
    speed: f32,
    base: Base<RigidBody2D>,
}

#[godot_api]
impl IRigidBody2D for Enemy {
    fn init(base: Base<RigidBody2D>) -> Self {
        Self {
            health: 5,
            speed: 100.0,
            base,
        }
    }
}

