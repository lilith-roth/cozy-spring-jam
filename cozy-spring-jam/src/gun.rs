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
    BulletCount,
    MultishotSpread,
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

    cooldown_animation_shown: bool,

    shooting: bool,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Gun {
    fn init(base: Base<Node2D>) -> Self {
        let mut attributes = Attributes::new();
        attributes
            .set_base(GunAttribute::Spread, 0.4)
            .set_base(GunAttribute::BulletCount, 1.0)
            .set_base(GunAttribute::Cooldown, 1.0)
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
            cooldown_animation_shown: false,
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
        self.play_animation("default", false);
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
        self.play_animation("shoot", false);
        if let Some(mut bullets) = BulletManager::for_node(self.base().upcast_ref()) {
            let pos = self.base().get_global_position();
            let self_rotation = self.base().get_global_rotation();

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

            let bullet_count = self.attr().get_int(GunAttribute::BulletCount);
            let multishot_spread = self.attr().get(GunAttribute::MultishotSpread);
            for i in 0..bullet_count {
                let direct_rotation =
                    self_rotation + (i as f32 / bullet_count as f32 - 0.5) * multishot_spread;
                let rotation = self.get_bullet_rotation(direct_rotation);

                bullets
                    .bind_mut()
                    .spawn_bullet(pos, rotation, params.clone());
            }
        }
    }

    pub fn set_shooting(&mut self, shooting: bool) {
        if !self.shooting && shooting {
            self.shoot();
        }
        self.shooting = shooting;
    }

    fn get_bullet_rotation(&mut self, direct: f32) -> f32 {
        let mut rng = RandomNumberGenerator::new_gd();
        let spread = self.attr().get(GunAttribute::Spread);
        let translation_diff = rng.randf_range(-spread, spread) / 2.0;
        direct + translation_diff
    }

    fn start_cooldown(&mut self) {
        self.cooldown_animation_shown = false;
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

    fn play_animation(&self, name: &str, reverse: bool) {
        if let Some(mut anim) = self.get_animation() {
            anim.stop();
            if reverse {
                anim.play_backwards_ex().name(name).done();
            } else {
                anim.play_ex().name(name).done();
            }
        }
    }

    fn on_animation_finished(&mut self) {
        if self.on_cooldown {
            if self.cooldown_animation_shown {
                self.play_animation("cooldown_transition", false);
            } else {
                self.play_animation("cooldown", false);
            }
        } else {
            self.play_animation("default", false);
        }
    }

    fn on_cooldown_finished(&mut self) {
        self.on_cooldown = false;
        self.play_animation("cooldown_transition", true);
        if self.shooting {
            self.shoot();
        }
    }
}
