#[path = "constraint.rs"] mod constraint;
pub use constraint::*;
use macroquad::prelude::*;
pub use std::collections::HashMap;



#[derive(Clone)]
pub struct Grid {
    pub cells: Vec<Vec<Vec<usize>>>,
    pub width: usize,
    pub height: usize,
    pub cellsize: f32,
}

impl Grid {
    pub fn new(width: usize, height: usize, cellsize: f32) -> Self {
        let mut cells = Vec::new();
        for _ in 0..(width as isize + 1) {
            let mut col = Vec::new();
            for _ in 0..(height as isize + 1) {
                col.push(Vec::new());
            }
            cells.push(col);
        }
        Self {
            cells: cells,
            width: width,
            height: height,
            cellsize: cellsize,
        }
    }
    pub fn update(&mut self, positions: &Vec<Vec2>) {
        for col in self.cells.iter_mut() {
            for cell in col.iter_mut() {
                cell.clear();
            }
        }
        for (uid, pos) in positions.iter().enumerate() {
            let i = (pos.x / self.cellsize) as usize;
            let j = (pos.y / self.cellsize) as usize;
            self.cells[i][j].push(uid);
        }
    }
    pub fn update_obj(&mut self, uid: usize, pos: Vec2) {
        let i = (pos.x / self.cellsize) as isize;
        let j = (pos.y / self.cellsize) as isize;
        for col in self.cells[((i - 1).max(0) as usize)..(i as usize + 1)].iter_mut() {
            for cell in col[((j - 1).max(0) as usize)..(j as usize + 1)].iter_mut() {
                match cell.iter().position(|&id| id == uid) {
                    Some(i) => { cell.remove(i); },
                    None => ()
                }
            }
        }
        self.cells[i as usize][j as usize].push(uid);
    }
    pub fn get(&self, x: usize, y: usize) -> &Vec<usize> {
        &self.cells[x][y]
    }
}