use crate::bullet::BulletAttribute;
use crate::enemy_drop::EnemyDrop;
use crate::gun::{Gun, GunAttribute};
use crate::player::Player;
use godot::builtin::{Array, Vector2, real};
use godot::classes::{
    Area2D, CharacterBody2D, IArea2D, ICharacterBody2D, IRigidBody2D, NavigationAgent2D, Node,
    Node2D, PackedScene, RandomNumberGenerator, RigidBody2D, Timer,
};
use godot::global::{godot_print, randi_range};
use godot::obj::{Base, Gd, NewGd, WithBaseField, WithUserSignals};
use godot::prelude::{GodotClass, godot_api};

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Enemy {
    #[export]
    health: i16,
    #[export]
    speed: f32,
    frames_since_facing_update: u16,

    #[export]
    gun: Option<Gd<Gun>>,

    #[export]
    loot_pool: Array<(Gd<PackedScene>)>,

    #[export]
    loot_chances: Array<u16>,

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
                y: root_scale.y,
            });
            self.frames_since_facing_update = 0;
        }
        if dir < 0.0 && root_scale.x > 0.0 {
            self.base_mut().set_scale(Vector2 {
                x: -1.0,
                y: root_scale.y,
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

                if current_pos.eq(&next_path_pos) {
                    return;
                }
                self.base_mut()
                    .set_velocity(current_pos.direction_to(next_path_pos) * nav_speed);
                self.base_mut().move_and_slide();
            }
        }
    }

    #[func]
    pub fn damage_enemy(&mut self, amount: i16) {
        self.health -= amount;
        if self.health <= 0 {
            let rng = randi_range(1, 100);
            for i in 0..self.loot_chances.len() {
                if (rng as u16)
                    < self
                        .loot_chances
                        .get(i)
                        .expect("Loot chance needs to defined!")
                {
                    let loot = self
                        .loot_pool
                        .get(i)
                        .expect("Every loot chance needs an assigned loot!")
                        .instantiate();
                    let mut loot_node: Gd<EnemyDrop> = loot
                        .expect("What could go wrong instantiating a scene?")
                        .cast();
                    loot_node.set_global_position(self.base_mut().get_global_position());
                    self.base_mut()
                        .get_parent()
                        .expect("We can access the parent, right?")
                        .add_child(&loot_node);
                }
            }
            self.base_mut().queue_free();
        }
    }

    const GUN_DISTANCE: f32 = 24.0;
    #[func]
    fn position_gun(&self, target_position: Vector2) {
        let self_pos = self.base().get_global_position();
        let facing = (target_position - self_pos).normalized();
        let facing_rot = f32::atan2(facing.y, facing.x);
        let gun_pos = Self::GUN_DISTANCE * facing;

        if let Some(mut gun) = self.gun.clone() {
            gun.set_position(gun_pos);
            gun.set_rotation(facing_rot);
        }
    }

    #[func]
    fn shoot(&mut self, target: Vector2) -> bool {
        self.position_gun(target);
        if let Some(mut gun) = self.get_gun() {
            if gun.bind().get_on_cooldown() {
                return false;
            }
            gun.bind_mut().shoot();
        }
        true
    }

    fn randomize_gun(&mut self) {
        let Some(mut gun) = self.get_gun() else {
            return;
        };
        let mut gun = gun.bind_mut();
        let mut rng = RandomNumberGenerator::new_gd();
        gun.attr()
            .set_base(GunAttribute::Spread, rng.randf_range(0.0, 0.8))
            .set_base(GunAttribute::Cooldown, rng.randf_range(0.1, 1.0))
            .set_base(GunAttribute::BulletCount, rng.randf_range(1.0, 5.0))
            .set_base(GunAttribute::MultishotSpread, rng.randf_range(0.4, 1.5))
            .set_base(
                GunAttribute::Bullets(BulletAttribute::Speed),
                rng.randf_range(200.0, 800.0),
            )
            .set_base(
                GunAttribute::Bullets(BulletAttribute::Power),
                rng.randf_range(0.5, 3.0),
            )
            .set_base(
                GunAttribute::Bullets(BulletAttribute::MaxBounces),
                rng.randf_range(0.0, 3.0),
            )
            .set_base(
                GunAttribute::Bullets(BulletAttribute::Lifetime),
                rng.randf_range(0.1, 3.0),
            );
    }
}

#[godot_api]
impl ICharacterBody2D for Enemy {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            health: 5,
            speed: 100.0,
            frames_since_facing_update: 0,
            gun: None,
            loot_pool: Default::default(),
            loot_chances: Default::default(),
            base,
        }
    }

    fn ready(&mut self) {
        self.randomize_gun();
        self.base_mut().set_y_sort_enabled(true);
    }
}

#[derive(GodotClass)]
#[class(base=Area2D)]
struct MeleeDetector {
    #[export]
    is_melee: bool,

    #[export]
    melee_damage: u16,

    base: Base<Area2D>,
}

#[godot_api]
impl MeleeDetector {
    fn on_body_entered(&mut self, node: Gd<Node2D>) {
        if !self.is_melee {
            return;
        }
        if node.is_class("Player") {
            godot_print!("Player damaged for {}", self.melee_damage);
            let mut player_node: Gd<Player> = node.cast();
            player_node
                .bind_mut()
                .damage_player(self.melee_damage as i16);
        }
    }
}

#[godot_api]
impl IArea2D for MeleeDetector {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            is_melee: false,
            melee_damage: 1,
            base: base,
        }
    }
    fn ready(&mut self) {
        self.signals()
            .body_entered()
            .connect_self(Self::on_body_entered);
    }
}
