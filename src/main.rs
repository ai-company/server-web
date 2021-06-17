mod ai_client;
mod database;
mod helper;
mod keys;
mod macros;
mod middleware;
mod route;
mod tokens;
mod util;

use std::{env, net::ToSocketAddrs};

use actix_web::{
    middleware::{Compress, NormalizePath},
    App, HttpServer,
};
use handlebars::Handlebars;
use jemallocator::Jemalloc;

use ai_client::AIClient;
use r2d2_sqlite::SqliteConnectionManager;

use crate::route::RootPath;

#[global_allocator]
static ALLOCATOR: Jemalloc = Jemalloc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let root_path = RootPath::new(
        String::from_utf8(
            env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .as_bytes()
                .into(),
        )
        .unwrap(),
    );

    let host_server = env::var("HOST_SERVER").unwrap_or("127.0.0.1".into());
    let host_danish = env::var("HOST_DANISH").unwrap_or("127.0.0.1".into());
    let host_english = env::var("HOST_ENGLISH").unwrap_or("127.0.0.1".into());

    let port_server = env::var("PORT_SERVER").unwrap_or("8000".into());
    let port_model_danish = env::var("PORT_MODEL_DANISH").unwrap_or("9000".into());
    let port_model_english = env::var("PORT_MODEL_ENGLISH").unwrap_or("9001".into());

    let addr_server = format!("{}:{}", host_server, port_server)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    let addr_danish = format!("{}:{}", host_danish, port_model_danish)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    let addr_english = format!("{}:{}", host_english, port_model_english)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    let model_danish = AIClient::new(addr_danish);
    let model_english = AIClient::new(addr_english);

    println!(
        "PATH: {}\nADDR-SRV: {}\nADDR-mEN: {}\nADDR-mDK: {}\n",
        &root_path, &addr_server, &addr_english, &addr_danish
    );

    HttpServer::new(move || {
        let manager = SqliteConnectionManager::file("users.db");
        let pool = r2d2::Pool::new(manager).unwrap();

        if let Err(e) = database::initialize(pool.clone()) {
            panic!("Failed initialize database with error: {}", e);
        }

        let shared_context = route::AiModels {
            model_danish: model_danish.clone(),
            model_english: model_english.clone(),
        };

        let mut templater = Handlebars::new();
        templater
            .register_templates_directory(".html", "template/")
            .unwrap();

        let asset_helper = util::AssetHelper {
            v_major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            v_minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
            v_patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
        };

        templater.register_helper("asset", Box::new(asset_helper));

        App::new()
            .wrap(Compress::default())
            .wrap(NormalizePath::default())
            .data(shared_context)
            .data(pool)
            .data(templater)
            .data(root_path.clone())
            .service(route::router())
    })
    .bind(addr_server)?
    .run()
    .await
}
