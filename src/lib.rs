
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio;
extern crate uuid;
extern crate mimalloc;
extern crate reqwest;
extern crate winit;

extern crate image;
extern crate gfx_auxil;

use mimalloc::MiMalloc;

pub mod home;
pub mod renderer;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
