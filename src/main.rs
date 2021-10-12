
#![recursion_limit = "256"]

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dss::app::App::new().await?.run()?;
    Ok(())
}
