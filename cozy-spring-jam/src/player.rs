mod health_hud;

use crate::gun::Gun;
use crate::player::health_hud::HealthHud;
use crate::room::Room;
use godot::builtin::{Vector2, real};
use godot::classes::{
    AnimatedSprite2D, Camera2D, CharacterBody2D, Control, ICharacterBody2D, Input, InputEvent,
    Node, PackedScene,
};
use godot::obj::{Base, Gd, WithBaseField};
use godot::prelude::{GodotClass, godot_api, load};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Orientation {
    Left,
    Right,
}

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Player {
    #[export]
    max_health: u16,
    #[export]
    health: i16,
    #[export]
    speed: f32,

    #[export]
    gun: Option<Gd<Gun>>,

    #[export]
    animation: Option<Gd<AnimatedSprite2D>>,

    orientation: Orientation,
    health_scene: Gd<PackedScene>,
    frames_since_last_healthbar_update: u16,
    base: Base<CharacterBody2D>,
}

#[godot_api]
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
            match i.cmp(&(self.health as u16)) {
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

        if movement_vec != Vector2::ZERO {
            self.play_animation("walk");
        } else {
            self.play_animation("default");
        }

        if movement_vec.x < 0.0 {
            self.orientation = Orientation::Left;
        } else if movement_vec.x > 0.0 {
            self.orientation = Orientation::Right;
        }

        let new_position = self.base_mut().get_global_position();
        let rooms = self
            .base_mut()
            .get_tree()
            .expect("Could not retrieve tree")
            .get_nodes_in_group("room");
        for i in 0..rooms.len() {
            let mut room: Gd<Room> = rooms.get(i).expect("Could not retrieve room!").cast();
            if new_position.x > room.get_global_position().x
                && new_position.x < room.get_global_position().x + 576.0
                && new_position.y > room.get_global_position().y
                && new_position.y < room.get_global_position().y + 352.0
            {
                // Generating adjacent rooms to the current room
                room.bind_mut().generate_adjacent_rooms();
                // Adjusting the camera for the current room
                let mut camera_node: Gd<Camera2D> = self
                    .base_mut()
                    .find_child("Camera2D")
                    .expect("Could not get camera node")
                    .cast();
                camera_node.set_global_position(room.get_global_position());
                break;
            }
        }
    }

    const GUN_DISTANCE: f32 = 24.0;

    fn position_gun(&self) {
        let Some(viewport) = self.base().get_viewport() else {
            return;
        };
        let self_pos = self.base().get_global_position();
        let mouse_pos = viewport.get_mouse_position();
        let facing = (mouse_pos - self_pos).normalized();
        let facing_rot = f32::atan2(facing.y, facing.x);
        let gun_pos = Self::GUN_DISTANCE * facing;

        if let Some(mut gun) = self.gun.clone() {
            gun.set_position(gun_pos);
            gun.set_rotation(facing_rot);
        }
    }

    #[func]
    pub fn damage_player(&mut self, amount: i16) {
        self.health -= amount;
        if self.health <= 0 {
            self.base_mut().get_tree().unwrap().quit();
        }
        if self.health > self.max_health as i16 {
            self.health = self.max_health as i16;
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
            animation: None,
            orientation: Orientation::Right,
            gun: None,
            base,
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        self.update_health_bar();
        self.position_gun();
        self.handle_walk_input();
        self.update_orientation();
        self.handle_shooting();
    }

    fn ready(&mut self) {
        self.play_animation("default");
    }
}

impl Player {
    fn play_animation(&self, name: &str) {
        if let Some(mut anim) = self.get_animation() {
            anim.play_ex().name(name).done();
        }
    }

    fn update_orientation(&self) {
        if let Some(mut anim) = self.get_animation() {
            anim.set_flip_h(self.orientation == Orientation::Left);
        }
    }

    fn handle_shooting(&mut self) {
        let input: Gd<Input> = Input::singleton();
        if let Some(mut gun) = self.get_gun() {
            gun.bind_mut()
                .set_shooting(input.is_action_pressed("shoot"));
        }
    }
}
