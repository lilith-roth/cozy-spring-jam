use godot::builtin::{real, Vector2};
use godot::classes::{CharacterBody2D, ICharacterBody2D, NavigationAgent2D, Node};
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
    fn move_towards_target(&mut self) {
        let nav_agent_raw: Option<Gd<Node>> = self.base_mut().find_child("NavigationAgent2D");
        match nav_agent_raw {
            None => panic!("An NPC needs a NavigationAgent2D for navigation!"),
            Some(nav_agent) => {
                let nav_speed = self.speed;
                let mut nav_agent_node: Gd<NavigationAgent2D> = nav_agent.cast();
                let current_pos: Vector2 = self.base_mut().get_global_position();
                let next_path_pos: Vector2 = nav_agent_node.get_next_path_position();
                self.base_mut().set_velocity(current_pos.direction_to(next_path_pos) * nav_speed);
                self.base_mut().move_and_slide();
            }
        }

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

