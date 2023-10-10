use glutin_window::GlutinWindow as Window;
use opengl_graphics::{OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderEvent, UpdateEvent};
use piston::window::WindowSettings;
mod renderer;
use renderer::Renderer;

mod hexcell_sim;
use hexcell_sim::{HexCellNetwork, Coordinate};
use hexcell_api::logging::{log, LogLevel, LogMessage, add_logger, LogCallback};

fn debug_log(level: LogLevel, msg: &str) -> ()
{
  println!("[{:?}] {}", level, msg);
  ()
}

fn main() {
  #[cfg(debug_assertions)]
  add_logger(debug_log);
  // Change this to OpenGL::V2_1 if not working.
  let opengl = OpenGL::V4_5;

  // Create a Glutin window.
  let mut window: Window = WindowSettings::new("hexcell", [1024, 1024])
      .graphics_api(opengl)
      .exit_on_esc(true)
      .build()
      .unwrap();

  let mut net: HexCellNetwork = HexCellNetwork::new();
  net.new_device(Coordinate { x: 0, y: 0});
  net.new_device(Coordinate { x: 0, y: 1});
  net.new_device(Coordinate { x: 1, y: 1});
  net.enable_connection(Coordinate { x: 0, y: 0 }, Coordinate { x: 0, y: 1 });

  // Create a new game and run it.
  let mut app = Renderer::new(opengl, Coordinate { x: 128, y: 128 });

  let mut events = Events::new(EventSettings::new());
  log(LogLevel::TRACE, "Entering main loop".into());
  while let Some(e) = events.next(&mut window) {
      net.update();
      if let Some(args) = e.render_args() {
          net.render(&mut app, &args);
      }

      if let Some(args) = e.update_args() {
          app.update(&args);
      }
  }
}
