#![recursion_limit = "256"]

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dss::hide_console_window();
    dss::app::App::new().await?.run()?;
    Ok(())
}
