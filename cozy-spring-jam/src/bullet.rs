use godot::{
    classes::{AnimatedSprite2D, IRigidBody2D, RigidBody2D},
    prelude::*,
};

#[derive(GodotClass)]
#[class(base=RigidBody2D)]
struct Bullet {
    #[export]
    animated_sprite: Option<Gd<AnimatedSprite2D>>,

    #[export]
    bounces: u32,

    #[export]
    bounce_velocity_preservation: f32,

    #[export]
    velocity: Vector2,

    #[export]
    lifetime: f32,

    #[var]
    age: f32,

    decayed: bool,

    base: Base<RigidBody2D>,
}

#[godot_api]
impl IRigidBody2D for Bullet {
    fn init(base: Base<RigidBody2D>) -> Self {
        Self {
            animated_sprite: None,
            bounces: 0,
            bounce_velocity_preservation: 1.0,
            velocity: Vector2::ZERO,
            lifetime: 2.0,
            age: 0.0,
            decayed: false,
            base,
        }
    }

    fn ready(&mut self) {
        self.play_animation("default");
        self.base_mut().set_contact_monitor(true);
        if let Some(mut anim) = self.animated_sprite.clone() {
            anim.signals()
                .animation_finished()
                .connect_obj(&*self, Self::on_animation_finished);
        }
        self.signals()
            .body_entered()
            .connect_self(Self::on_body_entered);
    }

    fn physics_process(&mut self, delta: f64) {
        let velocity = self.velocity;
        self.base_mut().set_linear_velocity(velocity);

        self.age += delta as f32;
        godot_print!("{}", self.age);
        if self.age > self.lifetime {
            self.decay();
        }
    }
}

#[godot_api]
impl Bullet {
    #[signal]
    fn impacted(pos: Vector2, node: Gd<Node>);

    #[signal]
    fn decayed(pos: Vector2);
}

impl Bullet {
    fn on_body_entered(&mut self, node: Gd<Node>) {
        todo!()
    }

    fn on_animation_finished(&mut self) {
        if self.decayed {
            self.base_mut().queue_free();
        }
    }

    fn position(&self) -> Vector2 {
        self.base().get_position()
    }

    fn decay(&mut self) {
        let pos = self.position();
        self.signals().decayed().emit(pos);
        self.play_animation("decay");
        self.decayed = true;
        if self.animated_sprite.is_none() {
            self.base_mut().queue_free();
        }
    }

    fn play_animation(&mut self, name: &str) {
        if let Some(anim) = &mut self.animated_sprite {
            anim.play_ex().name(name).done();
        }
    }
}
