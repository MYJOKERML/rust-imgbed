use axum::{
    extract::Extension,
    routing::{get, get_service},
    response::Html,
    http::StatusCode,
    Router,
};
use rand::seq::SliceRandom;
use tokio::io;
use std::sync::Arc;
// use tokio::fs::File;
use tower_http::services::ServeDir;

use lazy_static::lazy_static;

lazy_static! {
    static ref HOST: String = "0.0.0.0".to_string(); // 创建全局HOST变量
    static ref PORT: u16 = 80; // 创建全局PORT变量
}


#[derive(Debug)]
struct Image {
    path: String,
}

impl Image {
    fn new(path: String) -> Self {
        Image { path }
    }

    fn random_from_directory(directory: &str) -> Result<Self, io::Error> {
        let files: Vec<_> = std::fs::read_dir(directory)?
            .filter_map(|entry| {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        Some(path.to_string_lossy().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if let Some(random_path) = files.choose(&mut rand::thread_rng()) {
            Ok(Self::new(random_path.to_string()))
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "No images found"))
        }
    }
}


async fn random_image(Extension(images_directory): Extension<Arc<String>>) -> Html<String> {
    match Image::random_from_directory(&images_directory.as_str()) {
        Ok(image) => {
            let img_src = format!("http://{}:{}/{}", *HOST, *PORT, image.path);
            Html(format!("<img src=\"{}\" alt=\"Random Image\">", img_src))
        }
        Err(e) => {
            println!("Error: {:?}", e);
            Html("<p>No images found</p>".to_string())
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let images_directory = Arc::new("images".to_string());
    let current_dir = std::env::current_dir().unwrap();
    println!("current_dir: {:?}", current_dir);

    let app = Router::new()
        .route("/", get(random_image))
        .nest(
            "/images",
            get_service(ServeDir::new("images")).handle_error(|err| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("处理静态资源出错：{:?}", err),
                )
            }),
        )
        .layer(Extension(images_directory.clone()));

    println!("{:?}", images_directory);

    axum::Server::bind(&format!("{}:{}", *HOST, *PORT).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(())
}