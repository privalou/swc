use dotenv::dotenv;
use std::env;
use tokio::io;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    match env::args().nth(1) {
        Some(value) => {
            if value == "LOCAL" {
                println!("Using local env");
                dotenv().ok();
            }
        }
        None => {
            println!("Received no value from ENV param");
        }
    };

    let host = env::var("HOST").expect("Missing HOST env var");
    let port = env::var("PORT").expect("Missing PORT env var");

    let _server = server(host, port).await;

    Ok(())
}

async fn server(host: String, port: String) -> io::Result<()> {
    let mut server = tide::new();
    server
        .at("/health")
        .get(|_req: tide::Request<()>| async move { Ok("Ok".to_string()) });

    let http_server = server.listen(format!("{}:{}", host, port));
    http_server.await
}
