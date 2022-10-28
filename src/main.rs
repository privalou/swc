extern crate pretty_env_logger;

use dotenv::dotenv;

use std::env;
use std::net::ToSocketAddrs;
use swc::route::routes;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    if let Ok(value) = env::var("PROFILE") {
        if value == "LOCAL" {
            dotenv().ok();
        }
    }
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
    log::info!("Starting server");
    let host = env::var("HOST").expect("Missing HOST env var");
    let port = env::var("PORT").expect("Missing PORT env var");
    let server_details = format!("{}:{}", host, port);
    let server = server_details
        .clone()
        .to_socket_addrs()
        .expect("Unable to parse socket address")
        .next()
        .expect("Unable to parse socket address");
    let mongo_url = env::var("MONGO_URL").expect("Missing MONGO_URL env var");
    let client = mongodb::Client::with_uri_str(&mongo_url).await?;

    let api = routes(client);

    let routes = api.with(warp::log("groups"));
    warp::serve(routes).run(server).await;
    Ok(())
}
