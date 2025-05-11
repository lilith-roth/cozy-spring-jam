use std::collections::HashMap;

use godot::{
    classes::{AnimatedSprite2D, AudioStreamPlayer2D, RandomNumberGenerator, Timer},
    prelude::*,
};

use crate::{
    attribute::Attributes,
    bullet::{BulletAttribute, BulletManager, BulletParams},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GunAttribute {
    Cooldown,
    Spread,
    Bullets(BulletAttribute),
}

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct Gun {
    #[export]
    shoot_sfx: Option<Gd<AudioStreamPlayer2D>>,

    #[export]
    animation: Option<Gd<AnimatedSprite2D>>,

    #[export]
    cooldown_timer: Option<Gd<Timer>>,

    #[export]
    is_player_gun: bool,

    attributes: Attributes<GunAttribute>,

    #[var]
    on_cooldown: bool,

    shooting: bool,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Gun {
    fn init(base: Base<Node2D>) -> Self {
        let mut attributes = Attributes::new();
        attributes
            .set_base(GunAttribute::Spread, 0.4)
            .set_base(GunAttribute::Cooldown, 0.5)
            .set_base(GunAttribute::Bullets(BulletAttribute::Power), 1.0)
            .set_base(GunAttribute::Bullets(BulletAttribute::Lifetime), 1.0)
            .set_base(GunAttribute::Bullets(BulletAttribute::Speed), 200.0)
            .set_base(GunAttribute::Bullets(BulletAttribute::MaxBounces), 0.0)
            .set_base(
                GunAttribute::Bullets(BulletAttribute::BouncePowerPreservation),
                1.0,
            )
            .set_base(
                GunAttribute::Bullets(BulletAttribute::BounceSpeedPreservation),
                1.0,
            );
        Self {
            shoot_sfx: None,
            animation: None,
            cooldown_timer: None,
            is_player_gun: false,
            attributes,
            on_cooldown: false,
            shooting: false,
            base,
        }
    }

    fn ready(&mut self) {
        if let Some(mut anim) = self.get_animation() {
            anim.signals()
                .animation_finished()
                .connect_obj(&*self, Self::on_animation_finished);
        }
        if let Some(mut timer) = self.get_cooldown_timer() {
            timer.set_wait_time(self.attr().get(GunAttribute::Cooldown) as f64);
            timer
                .signals()
                .timeout()
                .connect_obj(&*self, Self::on_cooldown_finished);
        }
        self.play_animation("default");
    }
}

impl Gun {
    pub fn attr(&mut self) -> &mut Attributes<GunAttribute> {
        &mut self.attributes
    }

    pub fn shoot(&mut self) {
        if self.get_on_cooldown() {
            return;
        }
        self.start_cooldown();

        self.play_shoot();
        self.play_animation("shoot");
        if let Some(mut bullets) = BulletManager::for_node(self.base().upcast_ref()) {
            let pos = self.base().get_global_position();
            let rotation = self.get_bullet_rotation();

            let mut base_attributes = HashMap::new();
            for bullet_attr in BulletAttribute::ALL {
                base_attributes.insert(
                    *bullet_attr,
                    self.attr().get(GunAttribute::Bullets(*bullet_attr)),
                );
            }

            let params = BulletParams {
                base_attributes,
                is_player_bullet: self.is_player_gun,
            };

            bullets.bind_mut().spawn_bullet(pos, rotation, params);
        }
    }

    pub fn set_shooting(&mut self, shooting: bool) {
        if !self.shooting && shooting {
            self.shoot();
        }
        self.shooting = shooting;
    }

    fn get_bullet_rotation(&mut self) -> f32 {
        let mut rng = RandomNumberGenerator::new_gd();
        let spread = self.attr().get(GunAttribute::Spread);
        let translation_diff = rng.randf_range(-spread, spread) / 2.0;
        self.base().get_global_rotation() + translation_diff
    }

    fn start_cooldown(&mut self) {
        if let Some(mut timer) = self.get_cooldown_timer() {
            self.on_cooldown = true;
            timer.start();
        }
    }

    fn play_shoot(&self) {
        if let Some(mut sfx) = self.get_shoot_sfx() {
            sfx.play();
        }
    }

    fn play_animation(&self, name: &str) {
        if let Some(mut anim) = self.get_animation() {
            anim.stop();
            anim.play_ex().name(name).done();
        }
    }

    fn on_animation_finished(&mut self) {
        self.play_animation("default");
    }

    fn on_cooldown_finished(&mut self) {
        self.on_cooldown = false;
        if self.shooting {
            self.shoot();
        }
    }
}
