use axum::body::StreamBody;
use axum::extract::{BodyStream, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, put};
use axum::Router;
use futures_util::StreamExt;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::io::BufWriter;
use tokio_util::io::ReaderStream;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/download/:id", get(download))
        .route("/upload/:id", put(upload))
        .nest_service(
            "/",
            ServeDir::new("./static").not_found_service(ServeFile::new("./static/index.html")),
        );
    let addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn upload(Path(id): Path<String>, mut stream: BodyStream) -> impl IntoResponse {
    let path: String = format!("./files/{id}");
    let file = File::create(format!("./files/{id}")).await.unwrap();
    let mut writer = BufWriter::new(file);

    //Writing to file
    while let Some(Ok(chunk)) = stream.next().await {
        writer.write_all(&chunk).await.unwrap();
    }
    (StatusCode::CREATED, path)
}

async fn download(Path(id): Path<String>) -> impl IntoResponse {
    let path = format!("./files/{id}");
    //Checing if file exists
    let file = match File::open(path).await {
        Ok(file) => file,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return Err((StatusCode::NOT_FOUND, format!("File not found")))
        }
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error : {e}"))),
    };
    let body = StreamBody::new(ReaderStream::new(file));
    Ok((StatusCode::OK, body))
}
