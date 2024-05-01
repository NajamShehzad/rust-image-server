use actix_web::{web,get,App, HttpResponse, HttpServer, Responder, post};
use reqwest::Client;
use image::{ImageFormat, ImageOutputFormat, io::Reader as ImageReader};
use std::io::Cursor;
use base64::encode;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}



#[post("/convert-to-jpeg-from-url")]
async fn convert_to_jpeg_from_url(req_body: web::Json<serde_json::Value>) -> impl Responder {
    let url = req_body.get("url").and_then(|v| v.as_str()).unwrap_or_default();

    match get_image_base64_from_url(url).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}


async fn get_image_base64_from_url(url: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = Client::new();
    println!("Downloading image from: {}", url);
    let resp = client.get(url).send().await?;

    if resp.status().is_success() {
        let bytes = resp.bytes().await?;
        let cursor = Cursor::new(bytes.clone()); // Clone the bytes for potential Base64 encoding
        let img_reader = ImageReader::new(cursor);
        let image_format = img_reader.format();
        println!("Image format determined as {:?}", image_format);
     // Defaulting to Jpeg if format is unknown
        if let Some(format) = img_reader.format() {
            println!("Image format determined as {:?}", format);
            let image = img_reader.decode()?;
            match format {
                ImageFormat::Jpeg => {
                    // If the image is already JPEG, directly encode to Base64
                    let base64 = encode(&bytes); // Use the cloned bytes
                    println!("Encoding JPEG image to base64");
                    Ok(serde_json::json!({"success": true, "base64": base64, "imageType": "jpeg"}))
                },
                _ => {
                    // For non-JPEG images, convert to JPEG before encoding
                    let mut buffer = Cursor::new(Vec::new());
                    image.write_to(&mut buffer, ImageOutputFormat::Jpeg(80))?;
                    let base64 = encode(buffer.get_ref());
                    println!("Converted image to JPEG and encoded to base64");
                    Ok(serde_json::json!({"success": true, "base64": base64, "imageType": "jpeg"}))
                }
            }
        } else {
            println!("Could not determine the image format, attempting to decode as JPEG");
            // Try to force decode as JPEG if the format couldn't be determined
            let cursor = Cursor::new(bytes);
            match ImageReader::with_format(cursor, ImageFormat::Jpeg).decode() {
                Ok(image) => {
                    let mut buffer = Cursor::new(Vec::new());
                    println!("Forced JPEG decoding and encoded to base64");
                    image.write_to(&mut buffer, ImageOutputFormat::Jpeg(80))?;
                    let base64 = encode(buffer.get_ref());
                    Ok(serde_json::json!({"success": true, "base64": base64, "imageType": "jpeg"}))
                },
                Err(e) => {
                    println!("Failed to force decode image: {}", e);
                    Err(e.into())
                }
            }
        }
    } else {
        Err("Failed to download image or image is not available".into())
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = HttpServer::new(|| {
        App::new()
            .service(convert_to_jpeg_from_url)
            .service(hello)
    })
    .bind("127.0.0.1:8080")?;

    println!("Server running on http://127.0.0.1:8080"); // Console message after server starts
    server.run().await
}
