use glutin_window::GlutinWindow as Window;
use graphics::color::GRAY;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

use hexcell_api::display::LED_COUNT;
use crate::hexcell_sim::{HexCellNetwork, HexCellSim};

pub struct Renderer {
    pub gl: GlGraphics, // OpenGL drawing backend.
    pub rotation: f64  // Rotation for the square
}

impl Renderer {

    pub fn start_frame(&mut self, args: &RenderArgs)
    {
        use graphics::*;
        self.gl.draw(args.viewport(), |c, gl| {
            const GRAY: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
            clear(GRAY, gl);
        });
    }

    pub fn draw_sim(&mut self, args: &RenderArgs, sim: &HexCellSim, root_x:f64, root_y:f64)
    {
        use graphics::*;
        // Rings of leds [leds in each ring]
        const led_layout: [u32; 2] = [3, 6];
        assert!(led_layout.iter().sum::<u32>() >= LED_COUNT as u32);
        // Radius of each ring (leds occupy center of radial)
        const ring_radius: f64 = 64.0;
        const led_size: f64 = 32.0;

        let mut r:f64 = ring_radius / 2.0;
        let mut led_index:u8 = 0;
        for ring_count in led_layout
        {
            let delta_theta:f64 = (2.0*std::f64::consts::PI) / ring_count as f64;
            let mut theta:f64 = 0.0;
            for s in 1..ring_count
            {
                let led = sim.display.leds[led_index as usize];
                led_index += 1;
                let y = root_y + ((s as f64 * r) * theta.sin());
                let x = root_x + ((s as f64 * r) * theta.cos());

                let square = rectangle::square(0.0, 0.0, led_size as f64);

                self.gl.draw(args.viewport(), |c, gl| {

                    let transform = c
                        .transform
                        .trans(x, y)
                        .rot_rad(theta as f64);
                    let color: [f32; 4] = [led.r as f32 / 255.0, led.g as f32 / 255.0, led.b as f32 / 255.0 ,1.0];
                    // Draw a box rotating around the middle of the screen.
                    rectangle(color , square, transform, gl);
                });

                theta += delta_theta;
            }
            r += ring_radius;
        }
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
