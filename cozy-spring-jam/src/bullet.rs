use godot::{
    classes::{
        AnimatedSprite2D, AudioStreamPlayer2D, GpuParticles2D, IGpuParticles2D, IRigidBody2D,
        RigidBody2D, Timer,
    },
    prelude::*,
};

#[derive(GodotClass)]
#[class(base=RigidBody2D)]
struct Bullet {
    #[export]
    animated_sprite: Option<Gd<AnimatedSprite2D>>,

    #[export]
    bounce_sfx: Option<Gd<AudioStreamPlayer2D>>,

    #[export]
    bounces: u32,

    #[export]
    bounce_velocity_preservation: f32,

    #[export]
    bounce_power_preservation: f32,

    #[export]
    velocity: Vector2,

    #[export]
    lifetime: f32,

    #[export]
    power: f32,

    #[var]
    age: f32,

    dead: bool,

    base: Base<RigidBody2D>,
}

#[godot_api]
impl IRigidBody2D for Bullet {
    fn init(base: Base<RigidBody2D>) -> Self {
        Self {
            bounce_sfx: None,
            animated_sprite: None,
            bounces: 0,
            bounce_velocity_preservation: 1.0,
            bounce_power_preservation: 0.5,
            velocity: Vector2::ZERO,
            lifetime: 2.0,
            power: 1.0,
            age: 0.0,
            dead: false,
            base,
        }
    }

    fn ready(&mut self) {
        let velocity = self.velocity;
        self.base_mut().set_linear_velocity(velocity);

        self.play_animation("default");
        self.base_mut().set_contact_monitor(true);
        self.base_mut().set_max_contacts_reported(1);
        if let Some(mut anim) = self.animated_sprite.clone() {
            anim.signals()
                .animation_finished()
                .connect_obj(&*self, Self::free_if_dead);
        }
        self.signals()
            .body_entered()
            .connect_self(Self::on_body_entered);
    }

    fn physics_process(&mut self, delta: f64) {
        self.age += delta as f32;
        if self.age > self.lifetime {
            self.decay();
        }
    }
}

#[godot_api]
impl Bullet {
    #[signal]
    fn impacted(pos: Vector2, power: f32, node: Gd<Node>);

    #[signal]
    fn decayed(pos: Vector2);
}

impl Bullet {
    fn on_body_entered(&mut self, node: Gd<Node>) {
        let mut should_explode = false;
        if node.is_class("TileMapLayer") {
            if self.bounces == 0 {
                should_explode = true;
            } else {
                self.play_bounce();
                self.bounces -= 1;
            }
        } else {
            should_explode = true;
        }

        if should_explode {
            self.impact_explode(node);
        } else {
            self.impact(node);
        }
    }

    fn free_if_dead(&mut self) {
        if self.dead {
            self.base_mut().queue_free();
        }
    }

    fn position(&self) -> Vector2 {
        self.base().get_position()
    }

    fn impact(&mut self, node: Gd<Node>) {
        let pos = self.position();
        let power = self.get_power();
        self.signals().impacted().emit(pos, power, &node);
    }

    fn impact_explode(&mut self, node: Gd<Node>) {
        self.base_mut().hide();
        self.base_mut().set_linear_velocity(Vector2::ZERO);
        self.emit_explosion();
        self.impact(node);
        self.base_mut().queue_free();
    }

    fn decay(&mut self) {
        let pos = self.position();
        self.signals().decayed().emit(pos);
        self.play_animation("decay");
        self.dead = true;
        if self.animated_sprite.is_none() {
            self.base_mut().queue_free();
        }
    }

    fn play_animation(&mut self, name: &str) {
        if let Some(anim) = &mut self.animated_sprite {
            anim.play_ex().name(name).done();
        }
    }

    fn emit_explosion(&mut self) {
        if let Some(mut spawner) = BulletManager::for_node(self.base().upcast_ref()) {
            spawner.bind_mut().spawn_explosion(self.position());
        }
    }

    fn play_bounce(&mut self) {
        if let Some(mut sfx) = self.bounce_sfx.clone() {
            sfx.play();
        }
    }
}

#[derive(Debug, Clone)]
pub struct BulletParams {
    pub power: f32,
    pub speed: f32,
    pub bounces: u32,
    pub bounce_power_preservation: f32,
    pub bounce_velocity_preservation: f32,
    pub lifetime: f32,
}

impl Default for BulletParams {
    fn default() -> Self {
        Self {
            power: 1.0,
            speed: 400.0,
            bounces: 0,
            bounce_velocity_preservation: 1.0,
            bounce_power_preservation: 1.0,
            lifetime: 0.2,
        }
    }
}

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct BulletManager {
    bullet_scene: Gd<PackedScene>,
    bullet_explosion_scene: Gd<PackedScene>,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for BulletManager {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            bullet_scene: load(Self::BULLET_SCENE),
            bullet_explosion_scene: load(Self::BULLET_EXPLOSION_SCENE),
            base,
        }
    }

    fn ready(&mut self) {
        self.base_mut().set_y_sort_enabled(true);
    }
}

impl BulletManager {
    const BULLET_SCENE: &str = "res://scenes/bullet/bullet.tscn";
    const BULLET_EXPLOSION_SCENE: &str = "res://scenes/bullet/explosion.tscn";

    pub fn for_node(node: &Node) -> Option<Gd<BulletManager>> {
        let tree = node.get_tree()?;
        let scene = tree.get_current_scene()?;
        let result = scene.get_node_or_null("BulletManager").map(Gd::cast);
        if result.is_none() {
            godot_error!("No BulletManager found in this scene!");
        }
        result
    }

    pub fn spawn_bullet(&mut self, pos: Vector2, direction: Vector2, params: BulletParams) {
        let mut bullet: Gd<Bullet> = self
            .bullet_scene
            .instantiate()
            .expect("Failed to spawn bullet")
            .cast();
        bullet.set_position(pos);

        {
            let mut bullet_mut = bullet.bind_mut();
            bullet_mut.set_power(params.power);
            bullet_mut.set_velocity(params.speed * direction);
            bullet_mut.set_bounces(params.bounces);
            bullet_mut.set_bounce_velocity_preservation(params.bounce_velocity_preservation);
            bullet_mut.set_bounce_power_preservation(params.bounce_power_preservation);
            bullet_mut.set_lifetime(params.lifetime);
        }

        self.base_mut().add_child(&bullet);
    }

    pub fn spawn_explosion(&mut self, pos: Vector2) {
        let mut explosion: Gd<BulletExplosion> = self
            .bullet_explosion_scene
            .instantiate()
            .expect("Failed to spawn bullet explosion")
            .cast();
        explosion.set_position(pos);

        self.base_mut().add_child(&explosion);
    }
}

#[derive(GodotClass)]
#[class(base=GpuParticles2D, init)]
pub struct BulletExplosion {
    #[export]
    sfx: Option<Gd<AudioStreamPlayer2D>>,

    #[export]
    free_timer: Option<Gd<Timer>>,

    base: Base<GpuParticles2D>,
}

#[godot_api]
impl IGpuParticles2D for BulletExplosion {
    fn ready(&mut self) {
        self.base_mut().set_emitting(true);
        if let Some(mut timer) = self.free_timer.clone() {
            timer
                .signals()
                .ready()
                .connect_obj(&*self, Self::on_timer_ready);
        }
        if let Some(mut sfx) = self.sfx.clone() {
            sfx.play();
        }
    }
}

impl BulletExplosion {
    fn on_timer_ready(&mut self) {
        self.base_mut().queue_free();
    }
}
