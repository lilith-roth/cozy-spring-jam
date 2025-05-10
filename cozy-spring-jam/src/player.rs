mod health_hud;

use crate::bullet::{BulletManager, BulletParams};
use crate::player::health_hud::HealthHud;
use godot::builtin::{Vector2, real};
use godot::classes::{
    CharacterBody2D, Control, ICharacterBody2D, Input, InputEvent, Node, Node2D, PackedScene,
    TextureRect,
};
use godot::global::godot_print;
use godot::obj::{Base, Gd, WithBaseField};
use godot::prelude::{GodotClass, godot_api, load};
use std::cmp::Ordering;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
struct Player {
    #[export]
    max_health: u16,
    #[export]
    health: u16,
    #[export]
    speed: f32,
    #[export]
    bullet_spawner: Option<Gd<BulletManager>>,

    health_scene: Gd<PackedScene>,
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

    const BULLET_SPAWN_DISTANCE: f32 = 24.0;

    fn shoot(&self) {
        let Some(viewport) = self.base().get_viewport() else {
            return;
        };
        let self_pos = self.base().get_position();
        let mouse_pos = viewport.get_mouse_position();
        let bullet_dir = (mouse_pos - self_pos).normalized();

        if let Some(mut spawner) = self.bullet_spawner.clone() {
            spawner.bind_mut().spawn_bullet(
                self_pos + Self::BULLET_SPAWN_DISTANCE * bullet_dir,
                bullet_dir,
                BulletParams {
                    bounces: 5,
                    lifetime: 5.0,
                    power: 3.0,
                    ..BulletParams::default()
                },
            );
        }
    }
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
        let health_scene = load::<PackedScene>("res://ui/hud/health_hud.tscn");
        Self {
            max_health: 20,
            health: 20,
            speed: 150.0,
            health_scene,
            frames_since_last_healthbar_update: 1337,
            bullet_spawner: None,
            base,
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        self.update_health_bar();

        self.handle_walk_input()
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if event.is_action_pressed("shoot") {
            self.shoot();
        }
    }

    fn ready(&mut self) {}
}
