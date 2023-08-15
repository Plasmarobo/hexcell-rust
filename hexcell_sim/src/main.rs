use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
mod renderer;
use renderer::Renderer;

mod hexcell_sim;
use hexcell_sim::{HexCellSim, HexCellNetwork, Coordinate};

fn main() {
  // Change this to OpenGL::V2_1 if not working.
  let opengl = OpenGL::V4_5;

  // Create a Glutin window.
  let mut window: Window = WindowSettings::new("hexcell", [200, 200])
      .graphics_api(opengl)
      .exit_on_esc(true)
      .build()
      .unwrap();

  let mut net: HexCellNetwork = HexCellNetwork::new();
  net.new_device(Coordinate { x: 0, y: 0});
  net.new_device(Coordinate { x: 0, y: 1});

  // Create a new game and run it.
  let mut app = Renderer {
      gl: GlGraphics::new(opengl),
      rotation: 0.0
  };

  let mut events = Events::new(EventSettings::new());
  while let Some(e) = events.next(&mut window) {
      if let Some(args) = e.render_args() {
          net.render(&mut app, &args);
      }

      if let Some(args) = e.update_args() {
          app.update(&args);
      }
  }
}
