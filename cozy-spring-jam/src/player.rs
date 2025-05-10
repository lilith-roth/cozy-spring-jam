mod health_hud;

use godot::builtin::Vector2;
use godot::classes::{CharacterBody2D, ICharacterBody2D, Input};
use godot::obj::{Base, WithBaseField};
use godot::prelude::{GodotClass, godot_api};

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
struct Player {
    #[export]
    max_health: u16,
    #[export]
    health: u16,
    #[export]
    speed: f32,
    health_scene: Gd<PackedScene>,
    base: Base<CharacterBody2D>,
}

impl Player {

    fn draw_health_bar(&mut self) {
        for i in 0..self.max_health/2 {
            let new_health_scene = self.health_scene.instantiate().unwrap();

            self.base_mut().add_child(&new_health_scene);
        }
    }
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
        let health_scene = load::<PackedScene>("res://ui/hud/health_hud.tscn");
        let mut player = Self { max_health: 20, health: 20, speed: 150.0, health_scene, base };
        player.draw_health_bar();
        player
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
