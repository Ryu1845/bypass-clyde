use actix_web::{get, web, middleware, App, HttpResponse, HttpServer};
use awc::Client;
use serde::Deserialize;
use std::io::Cursor;
use env_logger::Env;

#[derive(Deserialize)]
pub struct QueryParameters {
    url: String,
}

fn create_gif(image_raw: image::RgbaImage) -> Result<Vec<u8>, image::ImageError> {
    // println!("Width: {}\nHeight: {}", image_raw.width(), image_raw.height());
    let width = image_raw.width();
    let height = image_raw.height();
    let size = width * height * 4;

    let mut empty_image = vec![0; size as usize];
    let empty_frame = gif::Frame::from_rgba_speed(width as u16, height as u16, &mut *empty_image, 30);
    // println!("Empty frame created");

    let mut image_pixels = image_raw.into_raw();
    let image_frame = gif::Frame::from_rgba_speed(width as u16, height as u16, &mut *image_pixels, 30);
    // println!("Image Frame Created");

    let frames: [gif::Frame; 2] = [empty_frame, image_frame];
    let mut gif = Vec::new();
    {
        let mut encoder = gif::Encoder::new(&mut gif, width as u16, height as u16, &[])
            .expect("Encoder Creation Error");
        // println!("Encoder created");
        for frame in &frames {
            encoder.write_frame(frame)
                .expect("Encoding Error");
        }
        // println!("Encoding Success");
    }
    Ok(gif)
}

async fn get_image(url: &str) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, awc::error::SendRequestError> {
    // println!("Image URL: {}", url);
    let client = Client::new();
    let response = client
        .get(url)
        .send()
        .await?
        .body()
        .limit(20_000_000)
        .await
        .expect("Request Error");
    // println!("Request Success");
    let data = response.as_ref();

    let reader = image::io::Reader::new(Cursor::new(data))
        .with_guessed_format()
        .expect("Format Error");
    let dyn_image = reader.decode()
        .expect("Decoding Error");
    let image = dyn_image.into_rgba8();
    // println!("Decoding Success");
    Ok(image)
}

#[get("/")]
async fn bypass_clyde(web::Query(info): web::Query<QueryParameters>) -> HttpResponse {
    let image = get_image(&info.url)
        .await
        .expect("Decoding Error");
    // println!("get_image success");
    let gif = create_gif(image)
        .expect("Gif Creation Error");
    // println!("create_gif success");
    HttpResponse::Ok()
        .header("Content-Type", "image/gif")
        .header("Cache-Control", "max-age=31536000")
        .body(gif)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Logger::new("%a %{User-Agent}i"))
            .service(bypass_clyde)
    })
    .bind("127.0.0.1:23423")?
    .run()
    .await
}
