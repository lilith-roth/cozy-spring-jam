use godot::{
    classes::{AnimatedSprite2D, AudioStreamPlayer2D, RandomNumberGenerator, Timer},
    prelude::*,
};

use crate::bullet::{BulletManager, BulletParams};

#[derive(GodotClass)]
#[class(base=Node2D, init)]
pub struct Gun {
    #[export]
    shoot_sfx: Option<Gd<AudioStreamPlayer2D>>,

    #[export]
    animation: Option<Gd<AnimatedSprite2D>>,

    #[export]
    cooldown: f64,

    #[export]
    spread: f32,

    #[export]
    cooldown_timer: Option<Gd<Timer>>,

    #[var]
    on_cooldown: bool,

    shooting: bool,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Gun {
    fn ready(&mut self) {
        if let Some(mut anim) = self.get_animation() {
            anim.signals()
                .animation_finished()
                .connect_obj(&*self, Self::on_animation_finished);
        }
        if let Some(mut timer) = self.get_cooldown_timer() {
            timer.set_wait_time(self.cooldown);
            timer
                .signals()
                .timeout()
                .connect_obj(&*self, Self::on_cooldown_finished);
        }
        self.play_animation("default");
    }
}

impl Gun {
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

            let bullet_dir = Vector2::new(rotation.cos(), rotation.sin());

            bullets.bind_mut().spawn_bullet(
                pos,
                bullet_dir,
                BulletParams {
                    bounces: 0,
                    lifetime: 0.5,
                    speed: 200.0,
                    power: 0.5,
                    ..BulletParams::default()
                },
            );
        }
    }

    pub fn set_shooting(&mut self, shooting: bool) {
        if !self.shooting && shooting {
            self.shoot();
        }
        self.shooting = shooting;
    }

    fn get_bullet_rotation(&self) -> f32 {
        let mut rng = RandomNumberGenerator::new_gd();
        let spread = rng.randf_range(-self.spread, self.spread) / 2.0;
        self.base().get_global_rotation() + spread
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
