//! Outil de debug : rend une scene depuis une camera donnee et ecrit le
//! resultat en PPM (format trivial, convertible en PNG avec n'importe quel
//! outil). Utile pour verifier le rendu sans navigateur ni fenetre native.
//!
//! Usage : `cargo run -p raycaster-core --example render_snapshot -- [x y angle_degres [ticks_immobile]] <sortie.ppm>`
//! `ticks_immobile` (optionnel, defaut 0) simule N pas de 0.2s sans bouger
//! avant le rendu - utile pour voir une porte finir de s'ouvrir.

use glam::Vec2;
use raycaster_core::{demo_level, Camera, Input, World};
use std::io::Write;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let (position, angle_degrees, ticks, output_path): (Vec2, f32, u32, String) = match args.as_slice() {
        [x, y, angle, ticks, output] => (
            Vec2::new(x.parse().expect("x invalide"), y.parse().expect("y invalide")),
            angle.parse().expect("angle invalide"),
            ticks.parse().expect("ticks invalide"),
            output.clone(),
        ),
        [x, y, angle, output] => (
            Vec2::new(x.parse().expect("x invalide"), y.parse().expect("y invalide")),
            angle.parse().expect("angle invalide"),
            0,
            output.clone(),
        ),
        [output] => (Vec2::new(8.0, 4.0), 0.0, 0, output.clone()),
        [] => (Vec2::new(8.0, 4.0), 0.0, 0, "snapshot.ppm".to_string()),
        _ => panic!("usage: render_snapshot [x y angle_degres [ticks_immobile]] <sortie.ppm>"),
    };

    let level = demo_level();
    let camera = Camera {
        position,
        dir: Vec2::from_angle(angle_degrees.to_radians()),
        eye_height: 1.0,
        fov: 66f32.to_radians(),
    };
    let mut world = World::new(level, camera);

    let stand_still = Input { forward: 0.0, strafe: 0.0, turn: 0.0 };
    for _ in 0..ticks {
        world.update(stand_still, 0.2);
    }

    let mut framebuffer = vec![0u32; WIDTH * HEIGHT];
    world.render(&mut framebuffer, WIDTH, HEIGHT);

    let file = std::fs::File::create(&output_path).expect("impossible de creer le fichier de sortie");
    let mut out = std::io::BufWriter::new(file);
    write!(out, "P6\n{WIDTH} {HEIGHT}\n255\n").unwrap();
    for &pixel in &framebuffer {
        let bytes = [
            ((pixel >> 16) & 0xFF) as u8,
            ((pixel >> 8) & 0xFF) as u8,
            (pixel & 0xFF) as u8,
        ];
        out.write_all(&bytes).unwrap();
    }

    println!("ecrit : {output_path}");
}
