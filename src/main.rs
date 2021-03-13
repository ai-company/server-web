extern crate chrono;
extern crate cookie;
extern crate lettre;
extern crate lettre_email;
extern crate mime;
extern crate nanoid;
extern crate time;
extern crate uriparse;
extern crate urldecode;

mod ai_client;
mod middleware;
mod route;
mod tokens;
mod web;

use ai_client::AIClient;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::{convert::TryFrom, sync::Arc};
use std::{env, net::SocketAddr};
use urldecode::decode as urldecode;
use web::StaticAssetStore;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() {
    let root_path = String::from_utf8(
        env::current_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .as_bytes()
            .into(),
    )
    .unwrap();

    let port_server = env::var("PORT_SERVER")
        .unwrap_or("8000".into())
        .parse()
        .unwrap();
    let addr_server = SocketAddr::from(([127, 0, 0, 1], port_server));
    let server = Server::bind(&addr_server);

    let port_model_danish = env::var("PORT_MODEL_DANISH")
        .unwrap_or("9000".into())
        .parse()
        .unwrap();
    let addr_danish = SocketAddr::from(([127, 0, 0, 1], port_model_danish));
    let model_danish = AIClient::new(addr_danish);

    let port_model_english = env::var("PORT_MODEL_ENGLISH")
        .unwrap_or("9001".into())
        .parse()
        .unwrap();
    let addr_english = SocketAddr::from(([127, 0, 0, 1], port_model_english));
    let model_english = AIClient::new(addr_english);

    let shared_context = Arc::new(route::SharedContext {
        root_path: root_path.clone(),
        asset_store: StaticAssetStore::new(&format!("{}/public/", root_path)),
        model_danish,
        model_english,
    });

    println!(
        "PATH: {}\nADDR-SRV: {}\nADDR-mEN: {}\nADDR-mDK: {}\n",
        &shared_context.root_path, addr_server, addr_english, addr_danish
    );

    let make_service = make_service_fn(move |_conn| {
        let shared_context = shared_context.clone();

        async move {
            Ok::<_, hyper::http::Error>(service_fn(move |mut req| {
                // extract and normalise path then reconstitute uri
                {
                    let path_and_query = req.uri().path_and_query().unwrap();
                    let mut path = uriparse::Path::try_from(path_and_query.path()).unwrap();
                    path.normalize(true);

                    let path = path.to_string().replace("//", "/");
                    if let Some(query) = path_and_query.query() {
                        *req.uri_mut() = format!("{}?{}", path.to_string(), query).parse().unwrap();
                    } else {
                        *req.uri_mut() = format!("{}", path.to_string()).parse().unwrap();
                    }
                }

                route::route(req, shared_context.clone())
            }))
        }
    });

    let executor = server
        .serve(make_service)
        .with_graceful_shutdown(shutdown_signal());

    if let Err(e) = executor.await {
        eprintln!("server error: {}", e);
    }
}
