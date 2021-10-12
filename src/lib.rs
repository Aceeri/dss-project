extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate mimalloc;
extern crate reqwest;
extern crate serde_json;
extern crate tokio;
extern crate uuid;
extern crate winit;

extern crate image;

use mimalloc::MiMalloc;

pub mod home;
pub mod renderer;
pub mod util;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
