use godot::classes::{Area2D, CharacterBody2D, IArea2D};
use godot::obj::Base;
use godot::prelude::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(base=Area2D)]
struct Bullet {
    #[export]
    health: u16,
    #[export]
    speed: f32,
    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for Bullet {
    
    fn init(base: Base<Area2D>) -> Self {
        Self {
            health: 1,
            speed: 200.0,
            base,
        }
    }
}