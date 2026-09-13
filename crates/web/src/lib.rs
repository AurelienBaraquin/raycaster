use glam::Vec2;
use raycaster_core::{Camera, Input, Map, World};
use wasm_bindgen::prelude::*;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

/// Instance de jeu exposee a JS. wasm-bindgen genere une classe JS autour de
/// cette struct : `new Game()` appelle `Game::new`, et `game.update(...)`/
/// `game.render()` appellent les methodes ci-dessous. Les champs restent prives
/// (jamais exposes a JS) et peuvent etre des types purement Rust (World, Vec<u32>)
/// sans que ca pose de probleme a wasm-bindgen.
#[wasm_bindgen]
pub struct Game {
    world: World,
    framebuffer: Vec<u32>,
    width: u32,
    height: u32,
    context: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Game {
        let map = Map::from_layout(
            "##########\n\
             #........#\n\
             #..####..#\n\
             #..#..#..#\n\
             #..#..#..#\n\
             #..####..#\n\
             #........#\n\
             ##########",
        );
        let camera = Camera::new(Vec2::new(1.5, 1.5));
        let world = World::new(map, camera);

        let window = web_sys::window().expect("no global window");
        let document = window.document().expect("no document");
        let canvas = document
            .get_element_by_id("canvas")
            .expect("no #canvas element")
            .dyn_into::<HtmlCanvasElement>()
            .expect("#canvas is not a canvas element");

        let width = canvas.width();
        let height = canvas.height();

        let context = canvas
            .get_context("2d")
            .expect("get_context failed")
            .expect("no 2d context")
            .dyn_into::<CanvasRenderingContext2d>()
            .expect("context is not CanvasRenderingContext2d");

        Game {
            world,
            framebuffer: vec![0u32; width as usize * height as usize],
            width,
            height,
            context,
        }
    }

    pub fn update(&mut self, forward: f32, strafe: f32, turn: f32, dt: f32) {
        self.world.update(Input { forward, strafe, turn }, dt);
    }

    pub fn render(&mut self) {
        self.world
            .render(&mut self.framebuffer, self.width as usize, self.height as usize);

        // core produit du 0x00RRGGBB, le Canvas 2D API attend du RGBA —
        // meme conversion que l'ancien smoke test `render_test_pattern`.
        let mut rgba = Vec::with_capacity(self.framebuffer.len() * 4);
        for px in &self.framebuffer {
            rgba.push(((px >> 16) & 0xFF) as u8);
            rgba.push(((px >> 8) & 0xFF) as u8);
            rgba.push((px & 0xFF) as u8);
            rgba.push(255);
        }

        let image_data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&rgba), self.width, self.height)
            .expect("failed to build ImageData");
        self.context
            .put_image_data(&image_data, 0.0, 0.0)
            .expect("put_image_data failed");
    }
}
