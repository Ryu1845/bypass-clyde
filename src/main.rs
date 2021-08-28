use actix_web::{client, get, web, App, HttpResponse, HttpServer};
use serde::Deserialize;
use std::io::Cursor;

#[derive(Deserialize)]
pub struct QueryParameters {
    url: String,
}

fn create_gif(image_raw: image::RgbaImage) -> Result<Vec<u8>, image::ImageError> {
    let empty_image = image::RgbaImage::new(image_raw.width(), image_raw.height());
    let empty_frame = image::Frame::new(empty_image);

    let duration = std::time::Duration::from_secs(300);
    let delay = image::Delay::from_saturating_duration(duration);
    let image_frame = image::Frame::from_parts(image_raw, 0, 0, delay);

    let frames: [image::Frame; 2] = [empty_frame, image_frame];
    let mut gif: Vec<u8> = Vec::new();
    {
        let mut encoder = image::codecs::gif::GifEncoder::new(&mut gif);
        encoder.encode_frames(frames)
            .expect("Encoding Error");
    }
    Ok(gif)
}

async fn get_image(url: &str) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, awc::error::SendRequestError> {
    let client = client::Client::new();
    let response = client
        .get(url)
        .send()
        .await?
        .body()
        .await
        .expect("Request Error");
    let data = response.as_ref();

    let reader = image::io::Reader::new(Cursor::new(data));
    let dyn_image = reader.decode()
        .expect("Decoding Error");
    let image = dyn_image.into_rgba8();
    Ok(image)
}

#[get("/")]
async fn bypass_clyde(web::Query(info): web::Query<QueryParameters>) -> HttpResponse {
    let image = get_image(&info.url)
        .await
        .expect("Decoding Error");
    let gif = create_gif(image)
        .expect("Gif Creation Error");
    HttpResponse::Ok().body(gif)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(bypass_clyde)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
