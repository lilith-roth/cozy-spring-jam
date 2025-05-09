use godot::builtin::math::FloatExt;
use godot::builtin::{real, Vector2};
use godot::classes::{CharacterBody2D, ICharacterBody2D, IRigidBody2D, Node, Node2D, RigidBody2D, SceneTree, Window};
use godot::global::godot_print;
use godot::obj::{Base, Gd, WithBaseField};
use godot::prelude::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
struct Enemy {
    #[export]
    health: u16,
    #[export]
    speed: f32,
    frames_since_facing_update: u16,
    moved_this_frame: bool,
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl Enemy {
    #[func]
    fn face_dir(&mut self, dir: f32) {
        let root_scale: Vector2 = self.base_mut().get_scale();

        if dir > 0.0 && root_scale.x < 0.0 {
            self.base_mut().set_scale(Vector2 {
                x: 1.0,
                y: root_scale.y
            });
            self.frames_since_facing_update = 0;
        }
        if dir < 0.0 && root_scale.x > 0.0 {
            self.base_mut().set_scale(Vector2 {
                x: -1.0,
                y: root_scale.y
            });
            self.frames_since_facing_update = 0;
        }
    }

    #[func]
    fn update_facing(&mut self) {
        self.frames_since_facing_update += 1;
        if self.frames_since_facing_update > 3 {
            let x_velocity: real = self.base_mut().get_velocity().x;
            self.face_dir(x_velocity);
        }
    }

    #[func]
    fn move_npc(&mut self, p_velocity: Vector2) {
        let current_velocity: Vector2 = self.base_mut().get_velocity();
        self.base_mut().set_velocity(Vector2 {
            x: current_velocity.x.lerp(p_velocity.x, 0.2),
            y: current_velocity.y.lerp(p_velocity.y, 0.2),
        });
        self.base_mut().move_and_slide();
    }
    
    fn post_physics_process(&mut self) {
        if !self.moved_this_frame {
            let current_velocity: Vector2 = self.base_mut().get_velocity();
            self.base_mut().set_velocity(Vector2 {
                x: current_velocity.x.lerp(Vector2::ZERO.x, 0.5),
                y: current_velocity.y.lerp(Vector2::ZERO.y, 0.5),
            });
        }
        self.moved_this_frame = false;
    }
}

#[godot_api]
impl ICharacterBody2D for Enemy {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            health: 5,
            speed: 100.0,
            frames_since_facing_update: 0,
            moved_this_frame: false,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        self.base_mut().call_deferred("post_physics_process", &[]);
    }
    
}

