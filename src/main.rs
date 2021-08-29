use actix_web::{App, HttpResponse, HttpServer, get, middleware, web};
use awc::Client;
use serde::Deserialize;
use std::io::Cursor;
use env_logger::Env;

#[derive(Deserialize)]
pub struct QueryParameters {
    url: String,
}

fn create_gif(image_raw: image::RgbaImage) -> Result<Vec<u8>, anyhow::Error> {
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
        let mut encoder = gif::Encoder::new(&mut gif, width as u16, height as u16, &[])?;
        // println!("Encoder created");
        for frame in &frames {
            encoder.write_frame(frame)?;
        }
        // println!("Encoding Success");
    }
    Ok(gif)
}

fn decode_image(image_raw: Vec<u8>) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, image::error::ImageError>{
    let reader = image::io::Reader::new(Cursor::new(image_raw))
        .with_guessed_format()?;
    let dyn_image = reader.decode()?;
    let image = dyn_image.into_rgba8();
    // println!("Decoding Success");
    Ok(image)
}

async fn get_image(url: &str) -> Result<Vec<u8>, actix_web::Error> {
    // println!("Image URL: {}", url);
    let client = Client::new();
    let mut response = client
        .get(url)
        .send()
        .await?;
    let response_body = response
        .body()
        .limit(20_000_000)
        .await?;
    // println!("Request Success");
    let data = response_body.as_ref();
    Ok(data.to_owned())
}

#[get("/")]
async fn bypass_clyde(web::Query(info): web::Query<QueryParameters>) -> Result<HttpResponse, actix_web::Error> {
    let url : String;
    if info.url.ends_with(".gif") {
        let len_url = &info.url.len();
        url = info.url[..len_url - 4].to_string();
    } else {
        url = info.url;
    };
    let image_raw = get_image(&url)
        .await?;
    // println!("get_image success");
    let image = decode_image(image_raw)
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    let gif = create_gif(image)
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    // println!("create_gif success");
    Ok(
        HttpResponse::Ok()
            .content_type("image/gif")
            .header("Cache-Control", "max-age=31536000")
            .header("Accept-Ranges", "bytes")
            .body(gif)
    )
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
