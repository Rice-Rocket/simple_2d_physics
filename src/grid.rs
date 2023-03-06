#[path = "constraint.rs"] mod constraint;
pub use constraint::*;
use macroquad::prelude::*;


#[derive(Clone)]
pub struct GridCell {
    pub x: f32,
    pub y: f32,
    pub cellsize: f32,
    pub objects: Vec<usize>,
    pub display_color: Color,
}

impl GridCell {
    pub fn new(x: f32, y: f32, cellsize: f32) -> Self {
        Self {
            x: x,
            y: y,
            cellsize: cellsize,
            objects: Vec::new(),
            display_color: Color::new(0.15, 0.15, 0.15, 1.0)
        }
    }
    pub fn draw(&self) {
        let dim = screen_width().min(screen_height());
        let y_diff = screen_height() - dim;
        let x_diff = screen_width() - dim;
        draw_rectangle_lines(self.x / 100. * dim + x_diff / 2., self.y / 100. * dim + y_diff / 2., self.cellsize / 100. * dim, self.cellsize / 100. * dim, 2., self.display_color);
    }
}


#[derive(Clone)]
pub struct Grid {
    pub cells: Vec<Vec<GridCell>>,
    pub width: usize,
    pub height: usize,
    pub cellsize: f32,
}

impl Grid {
    pub fn new(width: usize, height: usize, cellsize: f32) -> Self {
        let mut cells = Vec::new();
        for i in 0..(width + 1) {
            let mut col = Vec::new();
            for j in 0..(height + 1) {
                col.push(GridCell::new(i as f32 * cellsize, j as f32 * cellsize, cellsize));
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
                cell.objects.clear();
            }
        }
        for (uid, pos) in positions.iter().enumerate() {
            let i = (pos.x / self.cellsize) as usize;
            let j = (pos.y / self.cellsize) as usize;
            self.cells[i][j].objects.push(uid);
        }
    }
    pub fn update_obj(&mut self, uid: usize, pos: Vec2) {
        let i = (pos.x / self.cellsize) as isize;
        let j = (pos.y / self.cellsize) as isize;
        for col in self.cells[((i - 1).max(0) as usize)..(i as usize + 1)].iter_mut() {
            for cell in col[((j - 1).max(0) as usize)..(j as usize + 1)].iter_mut() {
                match cell.objects.iter().position(|&id| id == uid) {
                    Some(i) => { cell.objects.remove(i); },
                    None => ()
                }
            }
        }
        self.cells[i as usize][j as usize].objects.push(uid);
    }
    pub fn get(&self, x: usize, y: usize) -> &GridCell {
        &self.cells[x][y]
    }
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut GridCell {
        &mut self.cells[x][y]
    }
}