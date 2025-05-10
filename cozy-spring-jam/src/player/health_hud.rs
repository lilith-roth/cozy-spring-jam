use godot::classes::{ITextureRect, Texture2D, TextureRect};
use godot::obj::{Base, Gd, WithBaseField};
use godot::prelude::{GodotClass, godot_api};
use godot::tools::load;

#[derive(GodotClass)]
#[class(base=TextureRect)]
pub struct HealthHud {
    base: Base<TextureRect>,
}

#[godot_api]
impl ITextureRect for HealthHud {
    fn init(base: Base<TextureRect>) -> Self {
        Self { base }
    }

    fn process(&mut self, delta: f64) {
        let heart_state = self
            .base_mut()
            .get_child(0)
            .expect("HealthState node required!");

        let current_heart_state = heart_state.get_name().split("-")[1].to_string();
        match current_heart_state.as_str() {
            "FULL" => {
                let heart_full_image: Gd<Texture2D> =
                    load::<Texture2D>("res://assets/player/heart_full.png");
                self.base_mut().set_texture(&heart_full_image);
            }
            "HALF" => {
                let heart_half_image: Gd<Texture2D> =
                    load::<Texture2D>("res://assets/player/heart_half.png");
                self.base_mut().set_texture(&heart_half_image)
            }
            "EMPTY" => {
                let heart_empty_image: Gd<Texture2D> =
                    load::<Texture2D>("res://assets/player/heart_empty.png");
                self.base_mut().set_texture(&heart_empty_image)
            }
            &_ => panic!(
                "I'm disappointed in you to break this wonderfully crafted, absolutely not stupidly made system..."
            ),
        }
    }
}
