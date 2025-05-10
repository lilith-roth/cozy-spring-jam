use godot::classes::{ITextureRect, ResourcePreloader, Texture2D, TextureRect};
use godot::obj::{Base, Gd, WithBaseField};
use godot::prelude::{godot_api, GodotClass};
use godot::tools::load;

enum HealthHudStates {
    EMPTY,
    HALF,
    FULL
}

#[derive(GodotClass)]
#[class(base=TextureRect)]
struct HealthHud {
    health_hud_state: HealthHudStates,
    base: Base<TextureRect>,
}

#[godot_api]
impl ITextureRect for HealthHud {
    fn init(base: Base<TextureRect>) -> Self {
        Self {
            health_hud_state: HealthHudStates::FULL,
            base
        }
    }

    fn process(&mut self, delta: f64) {
        let heart_full_image: Gd<Texture2D> = load::<Texture2D>("res://assets/player/heart_full.png");
        let heart_half_image: Gd<Texture2D> = load::<Texture2D>("res://assets/player/heart_full.png");
        let heart_empty_image: Gd<Texture2D> = load::<Texture2D>("res://assets/player/heart_full.png");
        
        match self.health_hud_state{
            HealthHudStates::EMPTY => self.base_mut().set_texture(&heart_empty_image),
            HealthHudStates::HALF => self.base_mut().set_texture(&heart_half_image),
            HealthHudStates::FULL => self.base_mut().set_texture(&heart_full_image),
        }
    }
}