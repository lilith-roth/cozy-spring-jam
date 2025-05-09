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
    growth_noise_amplitude: f32,
    growth_noise_frequency: f32,
    grow_noise_octaves: i32,
    growth_noise_bias: f32,
    special_noise_frequency: f32,
    special_noise_amplitude: f32,
    noise_fractal_gain: f32,
    tree_growth_cutoff: f32,
    lone_tree_growth_cutoff: f32,
    lone_tree_special_cutoff: f32,
    grass_growth_cutoff: f32,
    tall_grass_growth_cutoff: f32,
}

impl Default for RoomGenParams {
    fn default() -> Self {
        Self {
            edge_growth: 1.5,
            center_growth: -1.0,
            growth_falloff: 0.6,
            growth_noise_amplitude: 4.0,
            growth_noise_frequency: 0.1,
            growth_noise_bias: -1.7,
            special_noise_frequency: 0.5,
            special_noise_amplitude: 3.0,
            grow_noise_octaves: 3,
            noise_fractal_gain: 0.6,
            tree_growth_cutoff: 0.6,
            lone_tree_growth_cutoff: 0.4,
            lone_tree_special_cutoff: 0.4,
            grass_growth_cutoff: 0.0,
            tall_grass_growth_cutoff: 0.2,
        }
    }
}

struct SpecialField<'a> {
    params: &'a RoomGenParams,
    noise: Gd<FastNoiseLite>,
}

impl<'a> SpecialField<'a> {
    fn new(seed: u32, params: &'a RoomGenParams) -> Self {
        let mut noise = FastNoiseLite::new_gd();
        noise.set_noise_type(NoiseType::VALUE);
        noise.set_seed(seed as i32);
        noise.set_frequency(params.special_noise_frequency);
        Self { params, noise }
    }

    fn get_special_factor(&self, x: usize, y: usize) -> f32 {
        let x = x as f32;
        let y = y as f32;

        self.params.special_noise_amplitude * self.noise.get_noise_2d(x, y)
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
        noise.set_frequency(params.growth_noise_frequency);
        noise.set_fractal_octaves(params.grow_noise_octaves);
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
        self.params.growth_noise_amplitude * self.noise.get_noise_2d(x, y)
            + self.params.growth_noise_bias
    }

    fn get_growth_factor(&self, x: usize, y: usize) -> f32 {
        let x = x as f32;
        let y = y as f32;

        (self.get_const_component(x, y) + self.get_noise_component(x, y)).clamp(-1.0, 1.0)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum FloorTile {
    #[default]
    None,
    Dirt,
    Grass,
    TallGrass,
}

#[derive(GodotClass)]
#[class(base=TileMapLayer, init)]
struct FloorLayer {
    #[export]
    dirt_terrain_set: i32,

    #[export]
    dirt_terrain: i32,

    #[export]
    tile_source: i32,

    #[export]
    grass_coords: Vector2i,

    #[export]
    tall_grass_coords: Vector2i,

    base: Base<TileMapLayer>,
}

impl FloorLayer {
    fn set_tiles(&mut self, grid: &Grid<FloorTile>) {
        self.base_mut().clear();

        let mut regions: HashMap<FloorTile, Array<Vector2i>> = HashMap::new();
        for (pos, tile) in grid {
            match tile {
                FloorTile::None => (),
                FloorTile::Grass => self.place_grass(pos),
                FloorTile::TallGrass => self.place_tall_grass(pos),
                FloorTile::Dirt => regions.entry(*tile).or_default().push_front(pos),
            }
        }

        for (tile, cells) in regions {
            match tile {
                FloorTile::Dirt => self.place_dirt(&cells),
                _ => (),
            }
        }
    }

    fn place_grass(&mut self, pos: Vector2i) {
        let source = self.tile_source;
        let coords = self.grass_coords;
        self.base_mut()
            .set_cell_ex(pos)
            .source_id(source)
            .atlas_coords(coords)
            .done();
    }

    fn place_tall_grass(&mut self, pos: Vector2i) {
        let source = self.tile_source;
        let coords = self.tall_grass_coords;
        self.base_mut()
            .set_cell_ex(pos)
            .source_id(source)
            .atlas_coords(coords)
            .done();
    }

    fn place_dirt(&mut self, cells: &Array<Vector2i>) {
        let terrain_set = self.dirt_terrain_set;
        let terrain = self.dirt_terrain;
        self.base_mut()
            .set_cells_terrain_connect(&cells, terrain_set, terrain);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum WallTile {
    #[default]
    Clear,
    Wall,
    LoneTree,
}

#[derive(GodotClass)]
#[class(base=TileMapLayer, init)]
struct WallsLayer {
    #[export]
    wall_terrain_set: i32,

    #[export]
    wall_terrain: i32,

    #[export]
    tile_source: i32,

    #[export]
    clear_coords: Vector2i,

    #[export]
    lone_tree_coords: Vector2i,

    base: Base<TileMapLayer>,
}

impl WallsLayer {
    fn set_tiles(&mut self, grid: &Grid<WallTile>) {
        self.base_mut().clear();

        let mut regions: HashMap<WallTile, Array<Vector2i>> = HashMap::new();
        for (pos, tile) in grid {
            match tile {
                WallTile::Clear => self.place_clear(pos),
                WallTile::LoneTree => self.place_lone_tree(pos),
                WallTile::Wall => regions.entry(*tile).or_default().push_front(pos),
            }
        }

        for (tile, cells) in regions {
            match tile {
                WallTile::Wall => self.place_walls(&cells),
                _ => (),
            }
        }
    }

    fn place_clear(&mut self, pos: Vector2i) {
        let source = self.tile_source;
        let coords = self.clear_coords;
        self.base_mut()
            .set_cell_ex(pos)
            .source_id(source)
            .atlas_coords(coords)
            .done();
    }

    fn place_lone_tree(&mut self, pos: Vector2i) {
        let source = self.tile_source;
        let coords = self.lone_tree_coords;
        self.base_mut()
            .set_cell_ex(pos)
            .source_id(source)
            .atlas_coords(coords)
            .done();
    }

    fn place_walls(&mut self, cells: &Array<Vector2i>) {
        let terrain_set = self.wall_terrain_set;
        let terrain = self.wall_terrain;
        self.base_mut()
            .set_cells_terrain_connect(&cells, terrain_set, terrain);
    }
}

#[derive(GodotClass)]
#[class(base=Node2D)]
struct Room {
    #[export]
    floor_layer: Option<Gd<FloorLayer>>,

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
            floor_layer: None,
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
        let special = SpecialField::new(seed, &self.params);

        let mut floor_grid: Grid<FloorTile> = Grid::new(self.width as usize, self.height as usize);
        self.generate_floor(&mut floor_grid, &growth);

        let mut wall_grid: Grid<WallTile> = Grid::new(self.width as usize, self.height as usize);

        self.place_special(&mut wall_grid, &growth, &special);
        self.place_walls(&mut wall_grid, &growth);

        if let Some(floor_layer) = &mut self.floor_layer {
            floor_layer.bind_mut().set_tiles(&floor_grid);
        }

        if let Some(walls_layer) = &mut self.walls_layer {
            walls_layer.bind_mut().set_tiles(&wall_grid);
        }
    }

    fn generate_floor(&self, grid: &mut Grid<FloorTile>, growth: &GrowthField) {
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Vector2i::new(x, y);
                let growth_factor = growth.get_growth_factor(x as usize, y as usize);

                if growth_factor >= self.params.tall_grass_growth_cutoff {
                    grid.set(pos, FloorTile::TallGrass);
                } else if growth_factor >= self.params.grass_growth_cutoff {
                    grid.set(pos, FloorTile::Grass);
                } else {
                    grid.set(pos, FloorTile::Dirt);
                }
            }
        }
    }

    fn place_special(
        &self,
        grid: &mut Grid<WallTile>,
        growth: &GrowthField,
        special: &SpecialField,
    ) {
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Vector2i::new(x, y);
                let growth_factor = growth.get_growth_factor(x as usize, y as usize);
                let special_factor = special.get_special_factor(x as usize, y as usize);

                if special_factor >= self.params.lone_tree_special_cutoff
                    && growth_factor >= self.params.lone_tree_growth_cutoff
                {
                    grid.set(pos, WallTile::LoneTree);
                }
            }
        }
    }

    fn place_walls(&self, grid: &mut Grid<WallTile>, growth: &GrowthField) {
        for y in 0..self.height {
            let is_vertical_edge = y == 0 || y == self.height - 1;
            for x in 0..self.width {
                let pos = Vector2i::new(x, y);
                let is_edge = is_vertical_edge || x == 0 || x == self.width - 1;
                let growth_factor = growth.get_growth_factor(x as usize, y as usize);

                if is_edge || growth_factor >= self.params.tree_growth_cutoff {
                    grid.set(pos, WallTile::Wall);
                }
            }
        }
    }
}
