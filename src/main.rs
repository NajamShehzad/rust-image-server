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
        println!("Image downloaded successfully");
        let bytes = resp.bytes().await?;
        let cursor = Cursor::new(bytes.clone()); // Clone the bytes for potential Base64 encoding
        print!("Cursor created");
        let img_reader = ImageReader::new(cursor);
        print!("Image reader created");
        let image_format = img_reader.format().unwrap_or(ImageFormat::Jpeg); // Defaulting to Jpeg if format is unknown
        print!("Image format determined");
        let image = img_reader.decode()?;
        println!("Image format: {:?}", image_format);
        if image_format == ImageFormat::Jpeg {
            let base64 = encode(&bytes); // Use the cloned bytes
            return Ok(serde_json::json!({"success": true, "base64": base64, "imageType": "jpeg"}));
        } else {
            let mut buffer = Cursor::new(Vec::new());
            image.write_to(&mut buffer, ImageOutputFormat::Jpeg(80))?;
            let base64 = encode(buffer.get_ref());
            return Ok(serde_json::json!({"success": true, "base64": base64, "imageType": "jpeg"}));
        }
    }

    Err("Failed to process image".into())
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
