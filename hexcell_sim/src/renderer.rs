

use std::cell;
use std::f64::consts::PI;
use std::path::Path;
use std::collections::{HashMap, HashSet};

use opengl_graphics::{GlGraphics, GlyphCache, Filter, TextureSettings, OpenGL, Texture};
use piston::input::{RenderArgs, UpdateArgs};
use graphics::{Image, rectangle, Context, Transformed, DrawState};

use hexcell_api::display::LED_COUNT;
use crate::hexcell_sim::{HexCellSim, Coordinate};
use assertions::const_assert;

macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)*) => (1usize + count!($($xs)*));
}

macro_rules! contentric_array {
    ($( $ring:expr),*) => {
        const N_RINGS:usize = (0usize + count!($( $ring )*));
        const N_LEDS: usize = (0usize $(+ $ring)*);
        const LED_LAYOUT: [u32; N_RINGS] = [$($ring,)*];
    }
}

contentric_array!(3, 6);
// Radius of each ring (leds occupy center of radial)
const LED_SIZE: f64 = 16.0;

pub struct Renderer<'R> {
    gl: GlGraphics, // OpenGL drawing backend.
    pub rotation: f64,  // Rotation for the square
    pub root: Coordinate,
    glyphs: GlyphCache<'R>,
    disconnected_tex: Texture,
    connect_tex: Texture,
    connected_tex: Texture,
    icon: Image,
    RING_RADIUS: f64,   
    HEX_RADIUS: f64,
    CELL_RADIUS: f64
}

impl<'R> Renderer<'R> {

    pub fn new(opengl: OpenGL, xy: Coordinate) -> Renderer<'R>
    {
        const_assert!(N_LEDS <= LED_COUNT);
        let texture_settings = TextureSettings::new().filter(Filter::Nearest);
        const ring_radius: f64 = 75.0;
        const hex_radius: f64 = ring_radius * 1.5;
        Renderer {
            gl: GlGraphics::new(opengl),
            rotation: 0.0,
            root: xy,
            glyphs: GlyphCache::new("assets/FiraSans-Regular.ttf", (), texture_settings).expect("Unable to load font"),
            disconnected_tex: Texture::from_path(Path::new("assets/disconnected.png"), &texture_settings).expect("Unable to load texture"),
            connect_tex: Texture::from_path(Path::new("assets/connect-action.png"), &texture_settings).expect("Unable to load texture"),
            connected_tex: Texture::from_path(Path::new("assets/connected.png"), &texture_settings).expect("Unable to load texture"),
            icon: Image::new().rect(rectangle::square(0.0, 0.0, 64.0)),
            RING_RADIUS: ring_radius,
            HEX_RADIUS: hex_radius,
            CELL_RADIUS: hex_radius * 1.2
        }
    }

    pub fn offset(&mut self, xy: Coordinate)
    {
        self.root = xy;
    }

    pub fn start_frame(&mut self, args: &RenderArgs)
    {
        use graphics::*;
        self.gl.draw(args.viewport(), |_c, gl| {
            const GRAY: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
            clear(GRAY, gl);
        });
    }

    fn hex_to_screen(root: Coordinate, cell_radius: f64, x: i32, y: i32) -> (f64, f64)
    {
        let rx = root.x as f64 + (x as f64 * 1.73 * cell_radius);
        let ry = root.y as f64 + (y as f64 * 2.0 * cell_radius) - if x.abs() % 2 == 1 {cell_radius} else { 0.0 };
        (rx, ry)
    }

    pub fn draw_hex(color: [f32;4], radius: f64, transform: [[f64; 3]; 2], gl: &mut GlGraphics)
    {
        const vertices_flat: [[f64; 2]; 6] = [
            [-1.0,   0.0],
            [-0.5,   0.87],
            [0.5,   0.87],
            [1.0,   0.0],
            [0.5,  -0.87],
            [-0.5,   -0.87],
        ];
        
        let transformed_vertices: Vec<[f64; 2]> = vertices_flat.into_iter().map(|vertex| [vertex[0] * radius, vertex[1] * radius]).collect();
        graphics::polygon(color, &transformed_vertices, transform, gl);
    }

    pub fn draw_hex_outline(color: [f32;4], transform: [[f64; 3]; 2], gl: &mut GlGraphics, size: f64)
    {
        // Find all hexagonal vertices
        const vertices_flat: [[f64; 2]; 6] = [
            [-1.0,   0.0],
            [-0.5,   0.87],
            [0.5,   0.87],
            [1.0,   0.0],
            [0.5,  -0.87],
            [-0.5,   -0.87],
        ];
        
        let transformed_vertices: Vec<[f64; 2]> = vertices_flat.into_iter().map(|vertex| [vertex[0] * size, vertex[1] * size]).collect();
        
        for i in 1..transformed_vertices.len()
        {
            let previous = transformed_vertices[i - 1];
            let current = transformed_vertices[i];
            graphics::line(color, 1.0, [previous[0], previous[1], current[0], current[1]], transform, gl);
        }
        let previous = transformed_vertices.last().unwrap();
        let current = transformed_vertices.first().unwrap();
        graphics::line(color, 1.0, [previous[0], previous[1], current[0], current[1]], transform, gl)
    }

    pub fn draw_grid(&mut self, args: &RenderArgs)
    {
        self.gl.draw(args.viewport(), |c, gl| {
            let color = [0.5, 0.5, 0.5, 0.5];
            
            let min_x: i32 = -1;
            let max_x: i32 = 10;
            let min_y: i32 = -1;
            let max_y: i32 = 10;

            for ix in min_x .. max_x
            {
                for iy in min_y .. max_y
                {
                    let (x, y) = Renderer::hex_to_screen(self.root, self.CELL_RADIUS, ix, iy);
                    let transform = c.transform
                    .trans(x, y);
                    Renderer::draw_hex_outline(color, transform, gl, self.CELL_RADIUS * 1.15)
                }
            }
        });
    }

    pub fn draw_sim(&mut self, args: &RenderArgs, sim: &HexCellSim, pos: Coordinate)
    {
        use graphics::*;
        
        // Rings of leds [leds in each ring]

        let (x, y) = Renderer::hex_to_screen(self.root, self.CELL_RADIUS, pos.x, pos.y);
        const white: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const black: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const border: f64 = 1.0;
        let mut r:f64 = self.RING_RADIUS / 2.0;
        let mut led_index:u8 = 0;
        
        self.gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform
            .trans(x, y)
            .rot_rad(0.0);
            Renderer::draw_hex(white, self.HEX_RADIUS, transform, gl);    
            //circle_arc([0.0,0.0,1.0,1.0], 4.0, 0.0, 2.0 * std::f64::consts::PI, rectangle::square(-5.0, -5.0, 10.0), transform, gl);
            let _ = text([0.0,0.0,0.0,1.0], 32, format!("{0}", sim.address).as_str(), &mut self.glyphs, transform.trans(-8.0, 8.0), gl);
            for ring_count in LED_LAYOUT
            {
                for i in 0..ring_count
                {
                    let led = sim.display.leds[led_index as usize];
                    led_index += 1;
                    let color: [f32; 4] = [led.r as f32 / 255.0, led.g as f32 / 255.0, led.b as f32 / 255.0 ,0.7];
                    
                    let theta:f64 = (i as f64 * (2.0 * PI) / ring_count as f64) - (2.0 * PI / 4.0);
                    let square = rectangle::square(-LED_SIZE/2.0, -LED_SIZE/2.0, LED_SIZE as f64);
                    let border_sq = [square[0] - border, square[1] - border, square[2] + (2.0 * border), square[3] + (2.0 * border)];
                    rectangle(black, border_sq, transform.trans(r* theta.cos(), r * theta.sin()).rot_rad(theta), gl);
                    rectangle(color, square, transform.trans(r * theta.cos(), r * theta.sin()).rot_rad(theta), gl);
                }
                r += self.RING_RADIUS * 0.5;
            }
        });
    }

    pub fn draw_connections(&mut self, args: &RenderArgs, connections: HashSet<(Coordinate, Coordinate)>)
    {
        self.gl.draw(args.viewport(), |c, gl| {
            for (source, dest) in connections
            {
                let (source_x, source_y) = Renderer::hex_to_screen(self.root, self.CELL_RADIUS, source.x, source.y);
                let (dest_x, dest_y) = Renderer::hex_to_screen(self.root, self.CELL_RADIUS, dest.x, dest.y);
                let (conn_x, conn_y) = ((source_x + dest_x) / 2.0, (source_y + dest_y) / 2.0);
                let transform = c.transform
                .trans(conn_x - 32.0, conn_y - 32.0);
                self.icon.draw(&self.connected_tex, &DrawState::default(), transform, gl);
            }
        });
    }

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GRAY: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = self.rotation;
        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GRAY, gl);

            let transform = c
                .transform
                .trans(x, y)
                .rot_rad(rotation)
                .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            rectangle(RED, square, transform, gl);
        });
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.rotation += 2.0 * args.dt;
    }
}
