extern crate pretty_env_logger;

use dotenv::dotenv;
use std::env;
use swc::routes;
use swc::state::State;
use tide::log;
use tokio::io;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();
    match env::args().nth(1) {
        Some(value) => {
            if value == "LOCAL" {
                log::info!("Using local env");
                dotenv().ok();
            }
        }
        None => {
            println!("Received no value from ENV param");
        }
    };
    log::info!("Starting server");
    let host = env::var("HOST").expect("Missing HOST env var");
    let port = env::var("PORT").expect("Missing PORT env var");
    let mongo_url = env::var("MONGO_URL").expect("Missing MONGO_URL env var");
    let _server = server(host, port, mongo_url).await;

    Ok(())
}

async fn server(host: String, port: String, mongo_url: String) -> io::Result<()> {
    let state = State::new(&mongo_url).await.expect("Can not create state");
    let mut server = tide::with_state(state);
    server
        .at("/group")
        .post(routes::create_group)
        .at("/health")
        .get(|_req: tide::Request<State>| async move { Ok("Ok".to_string()) });

    let http_server = server.listen(format!("{}:{}", host, port));
    http_server.await
}
