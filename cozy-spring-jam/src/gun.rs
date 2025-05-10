use godot::{classes::AudioStreamPlayer2D, prelude::*};

use crate::bullet::{BulletManager, BulletParams};

#[derive(GodotClass)]
#[class(base=Node2D, init)]
pub struct Gun {
    #[export]
    shoot_sfx: Option<Gd<AudioStreamPlayer2D>>,

    base: Base<Node2D>,
}

impl Gun {
    pub fn shoot(&self) {
        self.play_shoot();
        if let Some(mut bullets) = BulletManager::for_node(self.base().upcast_ref()) {
            let pos = self.base().get_position();
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
}
