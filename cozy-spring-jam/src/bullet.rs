use godot::builtin::Vector2;
use godot::classes::{Area2D, CharacterBody2D, IArea2D};
use godot::global::cubic_interpolate;
use godot::obj::{Base, WithBaseField};
use godot::prelude::{GodotClass, godot_api};

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct Bullet {
    #[export]
    health: u16,
    #[export]
    speed: f32,
    #[export]
    acceleration_factor: f32,
    #[export]
    direction: Vector2,
    #[export]
    velocity: Vector2,
    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for Bullet {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            health: 1,
            speed: 200.0,
            acceleration_factor: 1.05,
            direction: Vector2::ZERO,
            velocity: Vector2::ZERO,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let current_position = self.base_mut().get_global_position();
        let window_size = self.base_mut().get_tree().unwrap().get_root().unwrap().get_size_with_decorations();
        if current_position.x < 0f32 || current_position.x > window_size.x as f32 || current_position.y < 0f32 || current_position.y > window_size.y as f32 {
            self.base_mut().queue_free();
        }
        let velocity = Vector2 {
            x: self.velocity.x * self.acceleration_factor,
            y: self.velocity.y * self.acceleration_factor,
        };
        let current_position = self.base_mut().get_global_position();
        self.base_mut().set_global_position(Vector2 {
            x: current_position.x + velocity.x,
            y: current_position.y + velocity.y
        });
        self.velocity = velocity;
    }
}
