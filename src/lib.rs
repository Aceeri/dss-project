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

pub mod app;
pub mod grabber;
pub mod home;
pub mod menu;
pub mod renderer;
pub mod util;
