
#![recursion_limit = "256"]

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

pub mod app;
pub mod home;
pub mod grabber;
pub mod menu;
pub mod renderer;
pub mod util;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
