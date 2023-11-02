use actix_files::{Files, NamedFile};
use actix_web::{
    get,
    http::StatusCode,
    middleware::Logger,
    put,
    web::{self, Bytes},
    App, HttpResponse, HttpResponseBuilder, HttpServer,
};
use async_std::{fs::File, io::WriteExt};
use log::info;
use serde::Deserialize;
use std::fs::read_to_string;
use std::path::PathBuf;
#[derive(Deserialize, Clone)]
struct Config {
    max_file_size: u32, // Size in bytes
    file_dir: String,
}
impl Config {
    fn load(path: String) -> Result<Self> {
        Ok(toml::from_str::<Config>(&read_to_string(path)?)?)
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            max_file_size: 1_000_000_000, //1GB
            file_dir: "/Users/tymek/projects/Rust/fimshfreezer/files".to_string(),
        }
    }
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[actix_web::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let config = Config::load("config.toml".to_string()).unwrap_or_default();
    let _ = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::PayloadConfig::new(
                config.max_file_size.clone() as usize
            ))
            .wrap(Logger::default())
            .service(upload)
            .service(Files::new("/files", "./files"))
            .service(Files::new("/static", "./static"))
            .service(index)
    })
    .bind(("127.0.0.1", 80))?
    .run()
    .await?;
    Ok(())
}

#[derive(Deserialize)]
struct UploadQuery {
    name: String,
}
#[put("/upload")]
async fn upload(
    query: web::Query<UploadQuery>,
    bytes: Bytes,
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    let mut path: PathBuf = format!("./files/{}", query.name).into();
    let mut count = 0;
    // Guard against doing a silly
    if !path
        .canonicalize()
        .unwrap()
        .starts_with(config.file_dir.clone())
    {
        return Ok(HttpResponse::Unauthorized().into());
    }
    loop {
        path = PathBuf::from(format!(
            "./files/{}{}",
            match count {
                0 => String::new(),
                c => format!("({c})"),
            },
            query.name
        ));
        if !path.exists() {
            break;
        };
        count += 1;
    }
    File::create(path.clone()).await?.write_all(&bytes).await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).body(
        path.to_string_lossy()
            .to_string()
            .chars()
            .skip(1)
            .collect::<String>(),
    ))
}
#[get("/{_}*")]
async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("./static/index.html")?)
}
