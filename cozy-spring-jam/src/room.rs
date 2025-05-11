use crate::utils::Grid;
use godot::{
    classes::{
        FastNoiseLite, RandomNumberGenerator, TileMapLayer,
        fast_noise_lite::{FractalType, NoiseType},
    },
    prelude::*,
};
use std::collections::HashMap;

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
    exit_size: u32,
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
            exit_size: 2,
        }
    }
}

#[derive(Clone)]
struct RoomLayout {
    exit_top: bool,
    exit_bottom: bool,
    exit_left: bool,
    exit_right: bool,
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
    layout: &'a RoomLayout,
    width: f32,
    height: f32,
    noise: Gd<FastNoiseLite>,
    cache: HashMap<(usize, usize), f32>,
}

impl<'a> GrowthField<'a> {
    fn new(
        width: usize,
        height: usize,
        seed: u32,
        params: &'a RoomGenParams,
        layout: &'a RoomLayout,
    ) -> Self {
        let mut noise = FastNoiseLite::new_gd();
        noise.set_noise_type(NoiseType::PERLIN);
        noise.set_seed(seed as i32);
        noise.set_frequency(params.growth_noise_frequency);
        noise.set_fractal_octaves(params.grow_noise_octaves);
        noise.set_fractal_type(FractalType::RIDGED);
        noise.set_fractal_gain(params.noise_fractal_gain);
        Self {
            params,
            layout,
            width: width as f32 - 1.0,
            height: height as f32 - 1.0,
            noise,
            cache: HashMap::new(),
        }
    }

    fn get_edge_dist(&self, pos: Vector2) -> f32 {
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;

        let x_dist = (half_width - f32::abs(pos.x - half_width)).max(0.0);
        let y_dist = (half_height - f32::abs(pos.y - half_height)).max(0.0);

        f32::min(x_dist, y_dist)
    }

    fn is_on_critical_path(&self, pos_from_origin: Vector2, path: Vector2) -> bool {
        let path_dir = path.normalized();
        let dist_along_path = pos_from_origin.dot(path_dir);
        if dist_along_path.is_sign_negative() {
            return false;
        }

        let path_proportion = dist_along_path / path.length();
        let dist_to_center_line = (pos_from_origin - dist_along_path * path_dir).length();

        dist_to_center_line <= path_proportion * self.params.exit_size as f32
    }

    fn is_in_critical_section(&self, pos: Vector2) -> bool {
        let origin = Vector2::new(self.width, self.height) / 2.0;
        let pos_from_origin = pos - origin;

        let path_to_bottom = Vector2::new(0.0, self.height) / 2.0;
        let path_to_right = Vector2::new(self.width, 0.0) / 2.0;

        if self.layout.exit_top && self.is_on_critical_path(pos_from_origin, -path_to_bottom) {
            return true;
        }
        if self.layout.exit_bottom && self.is_on_critical_path(pos_from_origin, path_to_bottom) {
            return true;
        }
        if self.layout.exit_left && self.is_on_critical_path(pos_from_origin, -path_to_right) {
            return true;
        }
        if self.layout.exit_right && self.is_on_critical_path(pos_from_origin, path_to_right) {
            return true;
        }
        false
    }

    fn get_const_component(&self, pos: Vector2) -> f32 {
        let edge_dist = self.get_edge_dist(pos);

        (self.params.edge_growth * (1.0 - edge_dist * self.params.growth_falloff))
            .max(self.params.center_growth)
    }

    fn get_noise_component(&self, pos: Vector2) -> f32 {
        self.params.growth_noise_amplitude * self.noise.get_noise_2d(pos.x, pos.y)
            + self.params.growth_noise_bias
    }

    fn compute_growth_factor(&self, pos: Vector2) -> f32 {
        if self.is_in_critical_section(pos) {
            return f32::NEG_INFINITY;
        }

        (self.get_const_component(pos) + self.get_noise_component(pos)).clamp(-1.0, 1.0)
    }

    fn get_growth_factor(&mut self, x: usize, y: usize) -> f32 {
        let pos = Vector2::new(x as f32, y as f32);

        if let Some(cached_value) = self.cache.get(&(x, y)) {
            return *cached_value;
        }
        let value = self.compute_growth_factor(pos);
        self.cache.insert((x, y), value);
        value
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
pub struct Room {
    #[export]
    floor_layer: Option<Gd<FloorLayer>>,

    #[export]
    walls_layer: Option<Gd<WallsLayer>>,

    #[export]
    width: i32,

    #[export]
    height: i32,

    params: RoomGenParams,

    room_scene: Gd<PackedScene>,

    room_layout: Option<RoomLayout>,

    adjacent_rooms_generated: bool,

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
            room_scene: load("res://scenes/room_scene.tscn"),
            room_layout: None,
            adjacent_rooms_generated: false,
            base,
        }
    }

    fn ready(&mut self) {
        let mut rng = RandomNumberGenerator::new_gd();
        let layout = Option::from(RoomLayout {
            exit_top: rng.randi() % 2 == 0,
            exit_left: rng.randi() % 2 == 0,
            exit_bottom: rng.randi() % 2 == 0,
            exit_right: rng.randi() % 2 == 0,
        });
        self.generate(
            rng.randi(),
            &layout.clone().expect("Could not clone room layout"),
        );
        self.room_layout = layout;
    }
}

impl Room {
    fn generate(&mut self, seed: u32, layout: &RoomLayout) {
        let mut growth = GrowthField::new(
            self.width as usize,
            self.height as usize,
            seed,
            &self.params,
            &layout,
        );
        let special = SpecialField::new(seed, &self.params);

        let mut floor_grid: Grid<FloorTile> = Grid::new(self.width as usize, self.height as usize);
        self.generate_floor(&mut floor_grid, &mut growth);

        let mut wall_grid: Grid<WallTile> = Grid::new(self.width as usize, self.height as usize);

        self.place_special(&mut wall_grid, &mut growth, &special);
        self.place_walls(&mut wall_grid, &mut growth, &layout);

        if let Some(floor_layer) = &mut self.floor_layer {
            floor_layer.bind_mut().set_tiles(&floor_grid);
        }

        if let Some(walls_layer) = &mut self.walls_layer {
            walls_layer.bind_mut().set_tiles(&wall_grid);
        }
    }

    fn generate_floor(&self, grid: &mut Grid<FloorTile>, growth: &mut GrowthField) {
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
        growth: &mut GrowthField,
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

    fn place_walls(
        &self,
        grid: &mut Grid<WallTile>,
        growth: &mut GrowthField,
        layout: &RoomLayout,
    ) {
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Vector2i::new(x, y);
                let growth_factor = growth.get_growth_factor(x as usize, y as usize);

                if self.should_have_edge_wall(x, y, layout)
                    || growth_factor >= self.params.tree_growth_cutoff
                {
                    grid.set(pos, WallTile::Wall);
                }
            }
        }
    }

    fn should_have_edge_wall(
        &self,
        x: i32,
        y: i32,
        layout: &RoomLayout,
    ) -> bool {
        let center_dist_x = x - self.width / 2;
        let center_dist_y = y - self.height / 2;
        let max_dist = self.params.exit_size as i32 / 2;

        if layout.exit_top && center_dist_x.abs() <= max_dist && center_dist_y.is_negative() {
            return false;
        }
        if layout.exit_bottom && center_dist_x.abs() <= max_dist && center_dist_y.is_positive() {
            return false;
        }
        if layout.exit_left && center_dist_y.abs() <= max_dist && center_dist_x.is_negative() {
            return false;
        }
        if layout.exit_right && center_dist_y.abs() <= max_dist && center_dist_x.is_positive() {
            return false;
        }

        let is_edge = y == 0 || y == self.height - 1 || x == 0 || x == self.width - 1;
        is_edge
    }

    pub fn generate_adjacent_rooms(&mut self) {
        if self.adjacent_rooms_generated {
            return;
        }
        let layout_raw = self.room_layout.clone();
        match layout_raw {
            None => {
                godot_error!("Room layout not stored!")
            }
            Some(layout) => {
                let current_room_position = self.base_mut().get_global_position();
                godot_print!("Generating adjacent room to {:?}", current_room_position);
                if layout.exit_right {
                    let new_room = self.room_scene.instantiate();
                    let mut new_room_node: Gd<Room> =
                        new_room.expect("Could not be instantiated!").cast();
                    let new_room_position = Vector2 {
                        x: current_room_position.x + 576.0,
                        y: current_room_position.y,
                    };
                    if self.get_room_at_position(new_room_position).is_some() {
                        new_room_node.queue_free();
                    } else {
                        new_room_node.set_position(new_room_position);
                        self.base_mut()
                            .get_parent()
                            .expect("Could not get parent!")
                            .add_child(&new_room_node);
                        godot_print!(
                            "Generated new room at {:?}",
                            new_room_node.get_global_position()
                        );
                    }
                }
                if layout.exit_left {
                    let new_room = self.room_scene.instantiate();
                    let mut new_room_node: Gd<Room> =
                        new_room.expect("Could not be instantiated!").cast();
                    let new_room_position = Vector2 {
                        x: current_room_position.x - 576.0,
                        y: current_room_position.y,
                    };
                    if self.get_room_at_position(new_room_position).is_some() {
                        new_room_node.queue_free();
                    } else {
                        new_room_node.set_position(new_room_position);
                        self.base_mut()
                            .get_parent()
                            .expect("Could not get parent!")
                            .add_child(&new_room_node);
                        godot_print!(
                            "Generated new room at {:?}",
                            new_room_node.get_global_position()
                        );
                    }
                }
                if layout.exit_bottom {
                    let new_room = self.room_scene.instantiate();
                    let mut new_room_node: Gd<Room> =
                        new_room.expect("Could not be instantiated!").cast();
                    let new_room_position = Vector2 {
                        x: current_room_position.x,
                        y: current_room_position.y + 352.0,
                    };
                    if self.get_room_at_position(new_room_position).is_some() {
                        new_room_node.queue_free();
                    } else {
                        new_room_node.set_position(new_room_position);
                        self.base_mut()
                            .get_parent()
                            .expect("Could not get parent!")
                            .add_child(&new_room_node);
                        godot_print!(
                            "Generated new room at {:?}",
                            new_room_node.get_global_position()
                        );
                    }
                }
                if layout.exit_top {
                    let new_room = self.room_scene.instantiate();
                    let mut new_room_node: Gd<Room> =
                        new_room.expect("Could not be instantiated!").cast();
                    let new_room_position = Vector2 {
                        x: current_room_position.x,
                        y: current_room_position.y - 352.0,
                    };
                    if self.get_room_at_position(new_room_position).is_some() {
                        new_room_node.queue_free();
                    } else {
                        new_room_node.set_position(new_room_position);
                        self.base_mut()
                            .get_parent()
                            .expect("Could not get parent!")
                            .add_child(&new_room_node);
                        godot_print!(
                            "Generated new room at {:?}",
                            new_room_node.get_global_position()
                        );
                    }
                }
                self.adjacent_rooms_generated = true;
            }
        }
    }

    fn get_room_at_position(&mut self, pos: Vector2) -> Option<Gd<Room>> {
        let rooms = self
            .base_mut()
            .get_tree()
            .expect("Could not retrieve tree")
            .get_nodes_in_group("room");
        for i in 0..rooms.len() {
            let mut room: Gd<Room> = rooms.get(i).expect("Could not retrieve room!").cast();
            if pos.x > room.get_position().x
                && pos.x < room.get_position().x + 576.0
                && pos.y > room.get_position().y
                && pos.y < room.get_position().y + 352.0
            {
                godot_print!("Duplicated room! {:?}", pos);
                return Option::from(room);
            }
        }
        None
    }
}
