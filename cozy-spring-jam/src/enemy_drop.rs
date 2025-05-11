use crate::player::Player;
use godot::classes::{IRigidBody2D, Node, RigidBody2D};
use godot::global::godot_print;
use godot::obj::{Base, Gd, WithBaseField, WithUserSignals};
use godot::prelude::{GodotClass, godot_api};

#[derive(GodotClass)]
#[class(base=RigidBody2D)]
pub struct EnemyDrop {
    #[export]
    gained_health: i16,
    base: Base<RigidBody2D>,
}

#[godot_api]
impl EnemyDrop {
    fn on_body_entered(&mut self, node: Gd<Node>) {
        godot_print!("{:?}", node.get_class());
        if node.is_class("Player") {
            godot_print!("Player picked up");
            let mut player_node: Gd<Player> = node.cast();
            player_node
                .bind_mut()
                .damage_player(self.gained_health * -1);
            self.base_mut().queue_free();
        }
    }

    #[signal]
    fn picked_up(node: Gd<Node>);
}

#[godot_api]
impl IRigidBody2D for EnemyDrop {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            gained_health: 0,
            base,
        }
    }

    fn ready(&mut self) {
        self.base_mut().set_contact_monitor(true);
        self.base_mut().set_max_contacts_reported(1);
        self.signals()
            .body_entered()
            .connect_self(Self::on_body_entered);
    }
}
