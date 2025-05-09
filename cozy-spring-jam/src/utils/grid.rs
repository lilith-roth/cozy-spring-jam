use std::{iter::Enumerate, slice};

use godot::builtin::Vector2i;

#[derive(Debug, Clone)]
struct GridSize {
    width: usize,
    height: usize,
}

impl GridSize {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn pos_to_idx(&self, pos: Vector2i) -> usize {
        let mut norm_x = (pos.x.unsigned_abs() as usize) % self.width();
        if pos.x.is_negative() {
            norm_x = self.width() - norm_x;
        }
        let mut norm_y = (pos.y.unsigned_abs() as usize) % self.height();
        if pos.y.is_negative() {
            norm_y = self.height() - norm_y;
        }
        norm_y * self.width() + norm_x
    }

    fn idx_to_pos(&self, idx: usize) -> Vector2i {
        let x = idx % self.width;
        let y = idx / self.width;
        Vector2i::new(x as i32, y as i32)
    }
}

#[derive(Debug, Clone)]
pub struct Grid<T> {
    size: GridSize,
    entries: Vec<T>,
}

impl<T> Grid<T>
where
    T: Default + Clone,
{
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            size: GridSize { width, height },
            entries: vec![T::default(); width * height],
        }
    }
}

impl<T> Grid<T> {
    pub fn width(&self) -> usize {
        self.size.width()
    }

    pub fn height(&self) -> usize {
        self.size.height()
    }

    pub fn get(&self, pos: Vector2i) -> &T {
        self.entries
            .get(self.size.pos_to_idx(pos))
            .expect("error while accessing grid entries")
    }

    pub fn get_mut(&mut self, pos: Vector2i) -> &mut T {
        self.entries
            .get_mut(self.size.pos_to_idx(pos))
            .expect("error while accessing grid entries")
    }

    pub fn set(&mut self, pos: Vector2i, value: T) {
        *self.get_mut(pos) = value;
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            size: self.size.clone(),
            iter: self.entries.iter().enumerate(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            size: self.size.clone(),
            iter: self.entries.iter_mut().enumerate(),
        }
    }
}

impl<T> Default for Grid<T> {
    fn default() -> Self {
        Self {
            size: GridSize {
                width: 0,
                height: 0,
            },
            entries: Vec::new(),
        }
    }
}

pub struct Iter<'a, T> {
    size: GridSize,
    iter: Enumerate<slice::Iter<'a, T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (Vector2i, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let (idx, item) = self.iter.next()?;
        let pos = self.size.idx_to_pos(idx);
        Some((pos, item))
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, T> IntoIterator for &'a Grid<T> {
    type Item = <Iter<'a, T> as Iterator>::Item;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Grid<T> {
    type Item = <IterMut<'a, T> as Iterator>::Item;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct IterMut<'a, T> {
    size: GridSize,
    iter: Enumerate<slice::IterMut<'a, T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (Vector2i, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let (idx, item) = self.iter.next()?;
        let pos = self.size.idx_to_pos(idx);
        Some((pos, item))
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}
