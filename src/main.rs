mod ai_client;
mod middleware;
mod route;
mod tokens;
mod util;

use std::{env, net::ToSocketAddrs};

use actix_web::{App, HttpServer};
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

    let shared_context = route::SharedContext {
        root_path: root_path.clone(),
        model_danish,
        model_english,
    };

    println!(
        "PATH: {}\nADDR-SRV: {}\nADDR-mEN: {}\nADDR-mDK: {}\n",
        &shared_context.root_path,
        addr_server.clone(),
        addr_english,
        addr_danish
    );

    HttpServer::new(move || {
        App::new()
            .data(shared_context.clone())
            .service(route::router())
    })
    .bind(addr_server)?
    .run()
    .await
}
