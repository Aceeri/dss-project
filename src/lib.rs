
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio;
extern crate uuid;
extern crate mimalloc;
extern crate reqwest;

use mimalloc::MiMalloc;

mod home;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;