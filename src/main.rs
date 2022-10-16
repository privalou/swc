extern crate pretty_env_logger;

use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;
use swc::filters;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    if let Ok(value) = env::var("ENV") {
        if value == "LOCAL" {
            println!("Using local env");
            dotenv().ok();
        }
    }
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=todos=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "contacts=info");
    }
    if env::var_os("ENV").is_none() {
        env::set_var("ENV", "LOCAL");
    }
    pretty_env_logger::init();
    log::info!("Starting server");
    let host = env::var("HOST").expect("Missing HOST env var");
    let port = env::var("PORT").expect("Missing PORT env var");

    let server_details = format!("{}:{}", host, port);
    let server: SocketAddr = server_details
        .parse()
        .expect("Unable to parse socket address");
    let mongo_url = env::var("MONGO_URL").expect("Missing MONGO_URL env var");
    // GET /hello/warp => 200 OK with body "Hello, warp!"

    let client = mongodb::Client::with_uri_str(&mongo_url).await?;
    let api = filters::filters(client);

    let routes = api.with(warp::log("groups"));
    warp::serve(routes).run(server).await;
    Ok(())
}
