use glam::Vec2;
use minifb::{Key, Window, WindowOptions};
use raycaster_core::{demo_level, Camera, Input, World};
use std::time::Instant;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

fn main() {
    let level = demo_level();
    let camera = Camera::new(Vec2::new(1.5, 1.5));
    let mut world = World::new(level, camera);

    let mut window = Window::new("raycaster-native", WIDTH, HEIGHT, WindowOptions::default())
        .expect("failed to open window");

    let mut framebuffer = vec![0u32; WIDTH * HEIGHT];
    let mut last_frame = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f32();
        last_frame = now;

        let input = read_input(&window);
        world.update(input, dt);
        world.render(&mut framebuffer, WIDTH, HEIGHT);

        window
            .update_with_buffer(&framebuffer, WIDTH, HEIGHT)
            .expect("failed to update window buffer");
    }
}

// Commandes AZERTY (ZQSD). minifb resout les touches via xkb (le layout clavier
// actif), pas via la position physique brute : Key::W/Key::A correspondent donc
// aux caracteres 'w'/'a', absents en frappe directe sur un clavier AZERTY. Z et Q
// sont les equivalents AZERTY de W et A (meme position physique) ; S et D restent
// identiques sur les deux layouts.
fn read_input(window: &Window) -> Input {
    let mut forward = 0.0;
    let mut strafe = 0.0;
    let mut turn = 0.0;

    if window.is_key_down(Key::Z) {
        forward += 1.0;
    }
    if window.is_key_down(Key::S) {
        forward -= 1.0;
    }
    if window.is_key_down(Key::D) {
        strafe += 1.0;
    }
    if window.is_key_down(Key::Q) {
        strafe -= 1.0;
    }
    if window.is_key_down(Key::Right) {
        turn -= 1.0;
    }
    if window.is_key_down(Key::Left) {
        turn += 1.0;
    }

    Input { forward, strafe, turn }
}
