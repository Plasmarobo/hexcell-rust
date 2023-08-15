// This file hooks the application to the actual device model
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = stm32f103)]
mod app {
   #[shared]
   struct Shared {}

   #[local]
   struct Local {}

   #[init]
   fn init(_: init::Context) -> (Shared, Local)
   {
    debug::exit(debug::EXIT_SUCCESS);
    (Shared {}, Local {})
   }
}
