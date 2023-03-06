#[path = "constraint.rs"] mod constraint;
pub use constraint::*;
use macroquad::prelude::*;
pub use std::collections::HashMap;


#[derive(Clone)]
pub struct GridCell {
    pub x: f32,
    pub y: f32,
}

impl GridCell {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: x,
            y: y,
        }
    }
    pub fn draw(&self, cellsize: f32) {
        let dim = screen_width().min(screen_height());
        let y_diff = screen_height() - dim;
        let x_diff = screen_width() - dim;
        draw_rectangle_lines(self.x / 100. * dim + x_diff / 2., self.y / 100. * dim + y_diff / 2., cellsize / 100. * dim, cellsize / 100. * dim, 2., Color::new(0.15, 0.15, 0.15, 1.0));
    }
}


#[derive(Clone)]
pub struct Grid {
    pub cells: HashMap<(usize, usize), Vec<usize>>,
    pub width: usize,
    pub height: usize,
    pub cellsize: f32,
}

impl Grid {
    pub fn new(width: usize, height: usize, cellsize: f32) -> Self {
        Self {
            cells: HashMap::new(),
            width: width,
            height: height,
            cellsize: cellsize,
        }
    }
    pub fn update(&mut self, positions: &Vec<Vec2>) {
        self.cells.clear();
        for (uid, pos) in positions.iter().enumerate() {
            let i = (pos.x / self.cellsize) as usize;
            let j = (pos.y / self.cellsize) as usize;
            match self.cells.get_mut(&(i, j)) {
                Some(objs) => {
                    if !objs.contains(&uid) {
                        objs.push(uid);
                    }
                },
                None => {
                    self.cells.insert((i, j), vec![uid]);
                }
            }
        }
    }
    pub fn update_obj(&mut self, uid: usize, pos: Vec2) {
        let i = (pos.x / self.cellsize) as isize;
        let j = (pos.y / self.cellsize) as isize;
        for x in ((i - 1).max(0) as usize)..(i as usize + 1) {
            for y in ((j - 1).max(0) as usize)..(j as usize + 1) {
                match self.cells.get_mut(&(x, y)) {
                    Some(objs) => {
                        objs.retain(|x| *x != uid);
                    },
                    None => ()
                }
            }
        }
        match self.cells.get_mut(&(i as usize, j as usize)) {
            Some(objs) => {
                if !objs.contains(&uid) {
                    objs.push(uid);
                }
            },
            None => {
                self.cells.insert((i as usize, j as usize), vec![uid]);
            }
        };
    }
    pub fn get(&self, x: usize, y: usize) -> Option<&Vec<usize>> {
        self.cells.get(&(x, y))
    }
}