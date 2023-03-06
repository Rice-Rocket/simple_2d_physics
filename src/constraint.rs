use macroquad::prelude::*;


pub trait Constraint {
    fn get_new_pos(&self, position: Vec2, radius: f32) -> Option<Vec2>;
    fn draw(&self);
}


#[derive(Clone)]
pub struct CircleConstraint {
    pub position: Vec2,
    pub radius: f32,
}

impl CircleConstraint {
    pub fn new(position: Vec2, radius: f32) -> Box<Self> {
        Box::new(
            Self {
                position, radius
            }
        )
    }
}

impl Constraint for CircleConstraint {
    fn get_new_pos(&self, position: Vec2, radius: f32) -> Option<Vec2> {
        let to_pos = position - self.position;
        let dist = to_pos.length();
        if dist > (self.radius - radius) {
            let n = to_pos / dist;
            return Some(self.position + n * (self.radius - radius));
        } else {
            return None;
        }
    }
    fn draw(&self) {
        let dim = screen_width().min(screen_height());
        let y_diff = screen_height() - dim;
        let x_diff = screen_width() - dim;
        draw_circle_lines(self.position.x / 100. * dim + x_diff / 2., self.position.y / 100. * dim + y_diff / 2., self.radius / 100. * dim, 5., GRAY)
    }
}


#[derive(Clone)]
pub struct HalfSpace {
    pub normal: Vec2,
    pub point: Vec2,
}

impl HalfSpace {
    pub fn new(point: Vec2, normal: Vec2) -> Box<Self> {
        Box::new(
            Self {
                normal: normal.normalize(),
                point: point
            }
        )
    }
}

impl Constraint for HalfSpace {
    fn get_new_pos(&self, position: Vec2, radius: f32) -> Option<Vec2> {
        let dist = (position - self.point).dot(self.normal) - radius;
        if dist > 0.0 {
            return None;
        }
        return Some(position + (-dist * self.normal));
    }
    fn draw(&self) {
        let dim = screen_width().min(screen_height());
        let y_diff = screen_height() - dim;
        let x_diff = screen_width() - dim;
        let p1 = (self.point / 100. * dim) + vec2(x_diff / 2., y_diff / 2.);
        let p2 = ((self.point + (self.normal.perp().abs() * 100.)) / 100. * dim) + vec2(x_diff / 2., y_diff / 2.);
        draw_line(p1.x, p1.y, p2.x, p2.y, 5., GRAY);
    }
}