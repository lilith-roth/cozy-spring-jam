mod health_hud;

use crate::bullet::Bullet;
use crate::player::health_hud::HealthHud;
use godot::builtin::{Vector2, real};
use godot::classes::{
    CharacterBody2D, Control, ICharacterBody2D, Input, Node, Node2D, PackedScene, TextureRect,
};
use godot::global::godot_print;
use godot::obj::{Base, Gd, WithBaseField};
use godot::prelude::{GodotClass, godot_api, load};
use std::cmp::Ordering;
use std::ops::DerefMut;

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
    bullet_scene: Gd<PackedScene>,
    frames_since_last_healthbar_update: u16,
    base: Base<CharacterBody2D>,
}

impl Player {
    fn update_health_bar(&mut self) {
        self.frames_since_last_healthbar_update += 1;
        if self.frames_since_last_healthbar_update > 5 {
            self.frames_since_last_healthbar_update = 0;
            return;
        }
        let mut hud_node = self
            .base_mut()
            .find_child("Hud")
            .expect("Player needs a HUD node!")
            .find_child("HealthHuds")
            .expect("Player.Hud needs a HealthHuds node!");

        let expected_amount_health_containers = (f32::from(self.max_health / 2)).ceil();

        // Add new heart containers
        for i in hud_node.get_children().len()..expected_amount_health_containers as usize {
            godot_print!("{:?}", i);
            let node_str = format!("heart_container-{}", i);
            let new_health_scene = self.health_scene.instantiate().unwrap();
            let mut new_health_node: Gd<Control> = new_health_scene.cast();
            new_health_node.set_global_position(Vector2 {
                x: (28 * i + 8) as real,
                y: 8 as real,
            });
            new_health_node.set_name(&node_str);
            hud_node.add_child(&new_health_node);
        }

        // Remove too many heart containers
        for i in expected_amount_health_containers as usize..hud_node.get_children().len() {
            hud_node
                .get_child(i as i32)
                .expect("Something must've really went wrong dear")
                .queue_free()
        }

        // Update heart states
        for i in 0..self.max_health {
            let health_container: Gd<HealthHud> = hud_node
                .get_child((f32::from(i) / 2.0).floor() as i32)
                .expect(":thinking_face:")
                .cast();
            let mut heart_state: Gd<Node> = health_container
                .get_child(0)
                .expect("You need a HeartState named node with a HeartContainer!");
            let new_heart_state: &str;
            match i.cmp(&self.health) {
                Ordering::Less => new_heart_state = "-FULL",
                Ordering::Equal => new_heart_state = "-HALF",
                Ordering::Greater => new_heart_state = "-EMPTY",
            };
            let new_heart_state: String =
                heart_state.get_name().split("-")[0].to_string() + new_heart_state;
            heart_state.set_name(&new_heart_state);
        }
    }

    fn handle_walk_input(&mut self) {
        let input: Gd<Input> = Input::singleton();
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

    fn handle_shooting(&mut self) {
        let input: Gd<Input> = Input::singleton();
        let left = input.is_action_just_pressed("shoot_left");
        let right = input.is_action_just_pressed("shoot_right");
        let up = input.is_action_just_pressed("shoot_up");
        let down = input.is_action_just_pressed("shoot_down");
        if !left && !right && !up && !down {
            return;
        }
        if left && right {
            return;
        }
        if up && down {
            return;
        }
        let bullet_direction: Vector2 = Vector2 {
            x: if left {
                -1 as real
            } else if right {
                1 as real
            } else {
                0 as real
            },
            y: if up {
                -1 as real
            } else if down {
                1 as real
            } else {
                0 as real
            },
        };
        let mut bullet: Gd<Bullet> = self
            .bullet_scene
            .instantiate()
            .expect("Totally no reason to crash here right? :P")
            .cast();
        bullet.bind_mut().set_direction(bullet_direction);
        bullet.bind_mut().set_velocity(bullet_direction);
        self.base_mut().add_child(&bullet);
        self.base_mut().move_and_slide();
    }
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
        let health_scene = load::<PackedScene>("res://ui/hud/health_hud.tscn");
        let bullet_scene = load::<PackedScene>("res://scenes/bullet_scene.tscn");
        Self {
            max_health: 20,
            health: 20,
            speed: 150.0,
            health_scene,
            bullet_scene,
            frames_since_last_healthbar_update: 1337,
            base,
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        self.update_health_bar();

        self.handle_walk_input();
        self.handle_shooting();
    }

    fn ready(&mut self) {}
}
