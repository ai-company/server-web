mod ai_client;
mod middleware;
mod route;
mod tokens;
mod util;

use std::{env, net::ToSocketAddrs};

use actix_web::{middleware::Compress, App, HttpServer};
use handlebars::Handlebars;
use jemallocator::Jemalloc;

use ai_client::AIClient;

#[global_allocator]
static ALLOCATOR: Jemalloc = Jemalloc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let root_path = String::from_utf8(
        env::current_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .as_bytes()
            .into(),
    )
    .unwrap();

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
        let shared_context = route::SharedContext {
            root_path: root_path.clone(),
            model_danish: model_danish.clone(),
            model_english: model_english.clone(),
        };

        let mut templater = Handlebars::new();
        templater
            .register_templates_directory(".html", "template/")
            .unwrap();

        App::new()
            .wrap(Compress::default())
            .data(shared_context)
            .data(templater)
            .service(route::router())
    })
    .bind(addr_server)?
    .run()
    .await
}
