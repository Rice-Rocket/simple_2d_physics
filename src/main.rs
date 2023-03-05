use macroquad::prelude::*;
#[path = "space.rs"] mod space;
use space::*;
use std::f32::consts::PI;
use ::rand::{rngs::StdRng, Rng, SeedableRng, thread_rng};


fn spray(step: f32, scene: &mut Space, rng: &mut StdRng, origin: Vec2) {
    let theta = (step * 6.).sin() * PI / 2. + PI / 2.;
    
    let r = (step * 5.0).sin();
    let g = (step * 5.0 + 0.33 * 2.0 * PI).sin();
    let b = (step * 5.0 + 0.66 * 2.0 * PI).sin();

    let handle = scene.add_particle(origin, rng.gen_range(0.3..0.7));
    scene.set_velocity(handle, vec2(theta.cos() / screen_width() * 100., theta.sin() / screen_height() * 100.));
    scene.set_color(handle, Color::new(r * r, g * g, b * b, 1.0));
}



#[macroquad::main("2D Particle Physics Simulation")]
async fn main() {
    request_new_screen_size(640., 400.);
    next_frame().await;
    let mut dt: f32;
    let mut iteration = 0;
    let font = load_ttf_font("assets/Monaco.ttf").await.unwrap();

    let mut scene = Space::new();
    scene.set_gravity(vec2(0., 20.));
    scene.set_substeps(8);
    // scene.add_constraint(CircleConstraint::new(vec2(50., 50.), 45.));
    scene.add_constraint(HalfSpace::new(vec2(0., 90.), vec2(0., -1.)));
    scene.add_constraint(HalfSpace::new(vec2(0., 10.), vec2(0., 1.)));
    scene.add_constraint(HalfSpace::new(vec2(90., 0.), vec2(-1., 0.)));
    scene.add_constraint(HalfSpace::new(vec2(10., 0.), vec2(1., 0.)));

    let max_balls = 7200;
    let spray_origin = vec2(30., 40.);
    let mut rng = StdRng::seed_from_u64(15485748);
    let mut n_balls = 0;
    let mut paused = false;
    let mut dragging = false;
    let mut current_block = Vec::new();
    let particle_radius = 0.5;

    loop {
        iteration += 1;
        clear_background(BLACK);
        dt = get_frame_time();

        if n_balls < max_balls {
            n_balls += 1;
            spray(iteration as f32 / 800., &mut scene, &mut rng, spray_origin);
            spray(iteration as f32 / 800., &mut scene, &mut rng, spray_origin + vec2(40., 0.));
        }
        if is_key_pressed(KeyCode::Space) {
            paused = !paused
        }
        if is_mouse_button_down(MouseButton::Left) && (iteration % 3 == 0) {
            dragging = true;
            paused = true;
            match scene.localize(vec2(mouse_position().0, mouse_position().1)) {
                Some(pos) => {
                    if !scene.is_colliding(pos, particle_radius) {
                        n_balls += 1;
                        let handle = scene.add_particle(pos, particle_radius);
                        current_block.push(handle);
                    }
                },
                None => ()
            }
        }
        if dragging && is_mouse_button_released(MouseButton::Left) {
            let r = ((iteration as f32 / 800.) * 5.0).sin();
            let g = ((iteration as f32 / 800.) * 5.0 + 0.33 * 2.0 * PI).sin();
            let b = ((iteration as f32 / 800.) * 5.0 + 0.66 * 2.0 * PI).sin();
            for uid in current_block.iter() {
                scene.set_color(*uid, Color::new(r, g, b, 1.0));
            }
            scene.add_block(current_block.clone(), 0.05);
            current_block.clear();
            dragging = false;
            paused = false;
        }

        if !paused { scene.update(dt) };
        scene.draw_debug();
        scene.draw();

        draw_text_ex(
            &format!("FPS: {}", get_fps()),
            10.0, 30.0, 
            TextParams {font: font, font_size: 24u16, color: GRAY, ..Default::default()}
        );
        draw_text_ex(
            &format!("Balls: {}", n_balls),
            10.0, 60.0, 
            TextParams {font: font, font_size: 24u16, color: GRAY, ..Default::default()}
        );
        next_frame().await
    }
}
