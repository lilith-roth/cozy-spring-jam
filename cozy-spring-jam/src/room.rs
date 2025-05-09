use std::collections::HashMap;

use crate::utils::Grid;
use godot::{classes::TileMapLayer, prelude::*};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
enum WallTile {
    #[default]
    Default = 0,
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
#[class(base=Node2D, init)]
struct Room {
    #[export]
    walls_layer: Option<Gd<WallsLayer>>,

    #[export]
    width: i32,

    #[export]
    height: i32,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Room {
    fn ready(&mut self) {
        self.generate(0);
    }
}

impl Room {
    fn generate(&mut self, _seed: u32) {
        let mut grid: Grid<Tile> = Grid::new(self.width as usize, self.height as usize);
        self.place_walls(&mut grid);
        if let Some(walls_layer) = &mut self.walls_layer {
            walls_layer.bind_mut().set_tiles(&grid);
        }
    }

    fn place_walls(&self, grid: &mut Grid<Tile>) {
        for x in 0..self.width {
            grid.set(Vector2i::new(x, 0), Tile::Wall(WallTile::Default));
            grid.set(
                Vector2i::new(x, self.height - 1),
                Tile::Wall(WallTile::Default),
            );
        }

        for y in 1..self.height - 1 {
            grid.set(Vector2i::new(0, y), Tile::Wall(WallTile::Default));
            grid.set(
                Vector2i::new(self.width - 1, y),
                Tile::Wall(WallTile::Default),
            );
        }
    }
}
