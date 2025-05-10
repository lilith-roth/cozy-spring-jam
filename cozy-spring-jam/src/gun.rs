use godot::{
    classes::{AnimatedSprite2D, AudioStreamPlayer2D},
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
        self.play_animation("default");
    }
}

impl Gun {
    pub fn shoot(&self) {
        self.play_shoot();
        self.play_animation("shoot");
        if let Some(mut bullets) = BulletManager::for_node(self.base().upcast_ref()) {
            let pos = self.base().get_global_position();
            let rotation = self.base().get_rotation();

            let bullet_dir = Vector2::new(rotation.cos(), rotation.sin());

            bullets.bind_mut().spawn_bullet(
                pos,
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
}
