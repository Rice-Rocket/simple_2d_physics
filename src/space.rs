#[path = "grid.rs"] mod grid;
pub use grid::*;

use itertools::{izip, iproduct, Itertools};
use macroquad::prelude::*;
use rayon::prelude::*;
use std::sync::{Arc, Mutex, RwLock};
use num_cpus;



pub struct Space {
    positions: Vec<Vec2>,
    positions_old: Vec<Vec2>,
    accelerations: Vec<Vec2>,
    radii: Vec<f32>,
    colors: Vec<Color>,

    links: Vec<(usize, usize)>,
    link_dists: Vec<f32>,
    link_strengths: Vec<f32>,
    grid: Grid,
    constraints: Vec<Box<dyn Constraint>>,

    n_objects: usize,
    dt_substeps: usize,
    gravity: Vec2,
}

impl Space {
    pub fn new() -> Self {
        let cellsize = 0.7 * 3.;
        Self {
            positions: Vec::new(),
            positions_old: Vec::new(),
            accelerations: Vec::new(),
            radii: Vec::new(),
            colors: Vec::new(),

            links: Vec::new(),
            link_dists: Vec::new(),
            link_strengths: Vec::new(),
            grid: Grid::new((100. / cellsize) as usize, (100. / cellsize) as usize, cellsize),
            constraints: Vec::new(),

            n_objects: 0,
            dt_substeps: 1,
            gravity: vec2(0., 0.),
        }
    }
    pub fn set_gravity(&mut self, gravity: Vec2) {
        self.gravity = gravity;
    }
    pub fn set_substeps(&mut self, substeps: usize) {
        self.dt_substeps = substeps;
    }
    pub fn localize(&mut self, pos: Vec2) -> Option<Vec2> {
        let smaller_dim = screen_height().min(screen_width());
        let x_shift = screen_width() - smaller_dim;
        let y_shift = screen_height() - smaller_dim;
        
        let mut normalized = (pos - vec2(x_shift / 2., y_shift / 2.)) / smaller_dim;
        normalized *= 100.;

        if (normalized.x > 100.) || (normalized.y > 100.) || (normalized.x < 0.) || (normalized.y < 0.) {
            return None;
        }
        return Some(normalized);
    }
    
    pub fn add_particle(&mut self, position: Vec2, radius: f32) -> usize {
        self.positions.push(position);
        self.positions_old.push(position);
        self.radii.push(radius);
        self.colors.push(WHITE);
        self.accelerations.push(vec2(0., 0.));
        self.n_objects += 1;
        self.n_objects - 1
    }
    pub fn add_constraint(&mut self, constraint: Box<dyn Constraint>) {
        self.constraints.push(constraint);
    }
    pub fn add_link(&mut self, p1: usize, p2: usize, strength: f32) {
        if (p1 >= self.n_objects) || (p2 >= self.n_objects) {
            panic!("Point out of range");
        }
        self.links.push((p1, p2));
        self.link_dists.push((self.positions[p2] - self.positions[p1]).length());
        self.link_strengths.push(strength);
    }
    pub fn link_exists(&self, p1: usize, p2: usize) -> bool {
        if self.links.contains(&(p1, p2)) || self.links.contains(&(p2, p1)) {
            return true;
        }
        return false;
    }
    pub fn add_block(&mut self, particles: Vec<usize>, link_strength: f32) {
        for i in 0..particles.len() {
            let mut nearest = [self.n_objects; 8];
            let uid = particles[i];
            for j in 0..particles.len() {
                if i == j {
                    continue;
                }
                let uid2 = particles[j];
                for k in 0..8 {
                    if (nearest[k] == self.n_objects) || ((self.positions[nearest[k]] - self.positions[uid]).length() > (self.positions[uid2] - self.positions[uid]).length()) {
                        nearest[k] = uid2;
                        break;
                    } 
                }
            }
            for near_id in nearest {
                if !self.link_exists(uid, near_id) && (near_id != self.n_objects) {
                    self.add_link(uid, near_id, link_strength);
                }
            }
        }
    }
    pub fn remove_particle(&mut self, handle: usize) {
        self.positions.remove(handle);
        self.positions_old.remove(handle);
        self.accelerations.remove(handle);
        self.radii.remove(handle);
        self.colors.remove(handle);
        self.n_objects -= 1;
        for i in (0..self.links.len()).rev() {
            if (self.links[i].0 == handle) || (self.links[i].1 == handle) {
                self.links.remove(i);
                self.link_dists.remove(i);
                self.link_strengths.remove(i);
            }
        }
    }
    pub fn clear(&mut self) {
        self.positions.clear();
        self.positions_old.clear();
        self.accelerations.clear();
        self.radii.clear();
        self.colors.clear();
        self.links.clear();
        self.link_dists.clear();
        self.link_strengths.clear();
        self.n_objects = 0;
    }

    pub fn is_inside(&mut self, p1: usize, p2: usize) -> bool {
        (self.positions[p2] - self.positions[p1]).length() < (self.radii[p1] + self.radii[p2])
    }
    pub fn is_colliding(&mut self, pos: Vec2, radius: f32) -> bool {
        for i in 0..self.n_objects {
            if (self.positions[i] - pos).length() < (radius + self.radii[i]) {
                return true;
            }
        }
        return false;
    }

    pub fn set_position(&mut self, handle: usize, position: Vec2) {
        let delta = self.positions[handle] - position;
        self.positions[handle] = position;
        self.positions_old[handle] += delta;
    }
    pub fn get_position(&mut self, handle: usize) -> Vec2 {
        self.positions[handle]
    }
    pub fn set_color(&mut self, handle: usize, color: Color) {
        self.colors[handle] = color;
    }
    pub fn set_velocity(&mut self, handle: usize, velocity: Vec2) {
        self.positions_old[handle] = self.positions[handle] - velocity;
    }
    pub fn set_acceleration(&mut self, handle: usize, acceleration: Vec2) {
        self.accelerations[handle] = acceleration;
    }
    pub fn accelerate(&mut self, handle: usize, force: Vec2) {
        self.accelerations[handle] += force;
    }

    pub fn update(&mut self, dt: f32) {
        let sub_dt = dt / self.dt_substeps as f32;
        for _ in 0..self.dt_substeps {
            self.apply_gravity();
            self.apply_constraints();
            self.apply_links();
            self.remove_outside();
            self.grid.update(&self.positions);
            self.apply_collisions();

            for (i, (pos, pos_old, accel)) in izip!(self.positions.iter_mut(), self.positions_old.iter_mut(), self.accelerations.iter_mut()).enumerate() {
                let v = *pos - *pos_old;
                *pos_old = *pos;
                *pos = *pos + v + *accel * sub_dt * sub_dt;
                *accel = vec2(0., 0.);
            }
        }
    }
    pub fn remove_outside(&mut self) {
        for i in (0..self.n_objects).rev() {
            if (self.positions[i].x < 0.0) || (self.positions[i].x >= 100.0) || (self.positions[i].y < 0.0) || (self.positions[i].y >= 100.0) {
                self.remove_particle(i);
            }
        }
    }
    pub fn apply_gravity(&mut self) {
        for i in 0..self.n_objects {
            self.accelerate(i, self.gravity);
        }
    }
    pub fn apply_constraints(&mut self) {
        for constraint in self.constraints.iter() {
            for (pos, radius) in self.positions.iter_mut().zip(self.radii.iter()) {
                match constraint.get_new_pos(*pos, *radius) {
                    Some(new_pos) => {
                        *pos = new_pos;
                    },
                    None => (),
                }
            }
        }
    }
    pub fn apply_links(&mut self) {
        // let mut removed_links = Vec::new();
        for _ in 0..3 {
            for i in (0..self.links.len()).rev() {
                let (p1, p2) = self.links[i];
                let axis = self.positions[p1] - self.positions[p2];
                let dist = axis.length();
                let n = axis / dist;
                let mut delta = self.link_dists[i] - dist;
                if delta > self.link_strengths[i] {
                    delta = self.link_strengths[i];
                    self.links.remove(i);
                    self.link_dists.remove(i);
                    self.link_strengths.remove(i);
                }
                self.positions[p1] += 0.5 * delta * n;
                self.positions[p2] += -0.5 * delta * n;
            }
        }
    }
    pub fn apply_collisions(&mut self) {
        let iterator = self.grid.cells.clone();
        for ((x, y), objects) in iterator {
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if (x as isize + dx < 0) || (x as isize + dx > self.grid.width as isize - 1) || (y as isize + dy < 0) || (y as isize + dy > self.grid.height as isize - 1) {
                        continue;
                    }
                    let other;
                    match self.grid.get((x as isize + dx) as usize, (y as isize + dy) as usize) {
                        Some(objs) => { other = objs.clone(); },
                        None => { continue; }
                    }

                    for i in objects.iter() {
                        for j in other.iter() {
                            if *i != *j {
                                let collision_axis = self.positions[*i] - self.positions[*j];
                                let center_dist = self.radii[*i] + self.radii[*j];
                                let dist = collision_axis.length();
                                if dist < center_dist {
                                    let n = collision_axis / dist;
                                    let delta = center_dist - dist;
                                    self.positions[*i] += 0.5 * delta * n;
                                    self.positions[*j] += -0.5 * delta * n;
                                    self.grid.update_obj(*i, self.positions[*i]);
                                    self.grid.update_obj(*j, self.positions[*j]);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn draw(&mut self) {
        let smaller_dim = screen_width().min(screen_height());
        let y_diff = screen_height() - smaller_dim;
        let x_diff = screen_width() - smaller_dim;
        for (pos, radius, color) in izip!(self.positions.iter(), self.radii.iter(), self.colors.iter()) {
            let projected = vec2(pos.x / 100. * smaller_dim, pos.y / 100. * smaller_dim);
            draw_circle(projected.x + x_diff / 2., projected.y + y_diff / 2., *radius / 100. * smaller_dim, *color);
        }
    }
    pub fn draw_debug(&mut self) {
        let smaller_dim = screen_width().min(screen_height());
        let y_diff = screen_height() - smaller_dim;
        let x_diff = screen_width() - smaller_dim;
        
        for (x, y) in self.grid.cells.keys() {
            draw_rectangle_lines((*x as f32 * self.grid.cellsize) / 100. * smaller_dim + x_diff / 2., (*y as f32 * self.grid.cellsize) / 100. * smaller_dim + y_diff / 2., self.grid.cellsize / 100. * smaller_dim, self.grid.cellsize / 100. * smaller_dim, 2., Color::new(0.15, 0.15, 0.15, 1.0));
        }
        for constraint in self.constraints.iter() {
            constraint.draw();
        }
        for (p1, p2) in self.links.iter() {
            let pos1 = self.positions[*p1] / 100. * smaller_dim;
            let pos2 = self.positions[*p2] / 100. * smaller_dim;
            draw_line(pos1.x + x_diff / 2., pos1.y + y_diff / 2., pos2.x + x_diff / 2., pos2.y + y_diff / 2., 2., GRAY)
        }
    }
}