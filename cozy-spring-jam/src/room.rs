use std::collections::HashMap;

use crate::utils::Grid;
use godot::{
    classes::{
        FastNoiseLite, RandomNumberGenerator, TileMapLayer,
        fast_noise_lite::{FractalType, NoiseType},
    },
    prelude::*,
};

#[derive(Debug, Clone)]
struct RoomGenParams {
    growth_falloff: f32,
    edge_growth: f32,
    center_growth: f32,
    noise_amplitude: f32,
    noise_frequency: f32,
    noise_octaves: i32,
    noise_bias: f32,
    noise_fractal_gain: f32,
    tree_cutoff: f32,
}

impl Default for RoomGenParams {
    fn default() -> Self {
        Self {
            edge_growth: 1.0,
            center_growth: -5.0,
            growth_falloff: 1.0,
            noise_amplitude: 4.0,
            noise_frequency: 0.1,
            noise_bias: -0.5,
            noise_octaves: 2,
            noise_fractal_gain: 0.3,
            tree_cutoff: 0.6,
        }
    }
}

struct GrowthField<'a> {
    params: &'a RoomGenParams,
    width: f32,
    height: f32,
    noise: Gd<FastNoiseLite>,
}

impl<'a> GrowthField<'a> {
    fn new(width: usize, height: usize, seed: u32, params: &'a RoomGenParams) -> Self {
        let mut noise = FastNoiseLite::new_gd();
        noise.set_noise_type(NoiseType::PERLIN);
        noise.set_seed(seed as i32);
        noise.set_frequency(params.noise_frequency);
        noise.set_fractal_octaves(params.noise_octaves);
        noise.set_fractal_type(FractalType::RIDGED);
        noise.set_fractal_gain(params.noise_fractal_gain);
        Self {
            params,
            width: width as f32 - 1.0,
            height: height as f32 - 1.0,
            noise,
        }
    }

    fn get_edge_dist(&self, x: f32, y: f32) -> f32 {
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;

        let x_dist = (half_width - f32::abs(x - half_width)).max(0.0);
        let y_dist = (half_height - f32::abs(y - half_height)).max(0.0);

        f32::min(x_dist, y_dist)
    }

    fn get_const_component(&self, x: f32, y: f32) -> f32 {
        let edge_dist = self.get_edge_dist(x, y);

        (self.params.edge_growth * (1.0 - edge_dist * self.params.growth_falloff))
            .max(self.params.center_growth)
    }

    fn get_noise_component(&self, x: f32, y: f32) -> f32 {
        self.params.noise_amplitude * self.noise.get_noise_2d(x, y) + self.params.noise_bias
    }

    fn get_growth_factor(&self, x: usize, y: usize) -> f32 {
        let x = x as f32;
        let y = y as f32;

        let noise = self.get_noise_component(x, y);
        godot_print!("{noise}");

        (self.get_const_component(x, y) + self.get_noise_component(x, y)).clamp(-1.0, 1.0)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
enum WallTile {
    #[default]
    Tree = 0,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum Tile {
    #[default]
    Empty,
    Wall(WallTile),
}

#[derive(GodotClass)]
#[class(base=TileMapLayer, init)]
struct WallsLayer {
    #[export]
    terrain_set: i32,

    base: Base<TileMapLayer>,
}

impl WallsLayer {
    fn set_tiles(&mut self, grid: &Grid<Tile>) {
        let mut regions: HashMap<WallTile, Array<Vector2i>> = HashMap::new();
        for (pos, tile) in grid {
            let Tile::Wall(tile) = tile else {
                continue;
            };
            regions.entry(*tile).or_default().push_front(pos);
        }

        self.base_mut().clear();
        let terrain_set = self.terrain_set;
        for (tile, cells) in regions {
            self.base_mut()
                .set_cells_terrain_connect(&cells, terrain_set, tile as i32);
        }
    }
}

#[derive(GodotClass)]
#[class(base=Node2D)]
struct Room {
    #[export]
    walls_layer: Option<Gd<WallsLayer>>,

    #[export]
    width: i32,

    #[export]
    height: i32,

    params: RoomGenParams,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Room {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            walls_layer: None,
            width: 0,
            height: 0,
            params: RoomGenParams::default(),
            base,
        }
    }

    fn ready(&mut self) {
        let mut rng = RandomNumberGenerator::new_gd();
        self.generate(rng.randi());
    }
}

impl Room {
    fn generate(&mut self, seed: u32) {
        let growth = GrowthField::new(
            self.width as usize,
            self.height as usize,
            seed,
            &self.params,
        );

        let mut grid: Grid<Tile> = Grid::new(self.width as usize, self.height as usize);

        self.place_walls(&mut grid, &growth);

        if let Some(walls_layer) = &mut self.walls_layer {
            walls_layer.bind_mut().set_tiles(&grid);
        }
    }

    fn place_walls(&self, grid: &mut Grid<Tile>, growth: &GrowthField) {
        for y in 0..self.height {
            let is_vertical_edge = y == 0 || y == self.height - 1;
            for x in 0..self.width {
                let is_edge = is_vertical_edge || x == 0 || x == self.width - 1;
                let growth_factor = growth.get_growth_factor(x as usize, y as usize);

                if is_edge || growth_factor >= self.params.tree_cutoff {
                    grid.set(Vector2i::new(x, y), Tile::Wall(WallTile::Tree));
                }
            }
        }
    }
}
