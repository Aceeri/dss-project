
use tokio::stream;

use dss::menu::{Menu, EventGrab, Collection, Tile};
use dss::renderer::{Renderer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let client = reqwest::Client::new();

    let http_grabber = dss::http_grabber::HttpGrabber::new();
    /*let image_url = "https://prod-ripcut-delivery.disney-plus.net/v1/variant/disney/3C33485A3043C22B8C89E131693E8B5B9306DAA4E48612A655560752977728A6/scale?format=jpeg&quality=90&scalingAlgorithm=lanczos3&width=500".to_owned();
    println!("{:?}", image_grabber.poll(image_url.clone()));
    println!("{:?}", image_grabber.poll(image_url.clone()));*/

    dss::app::App::new();

    Ok(())
}
