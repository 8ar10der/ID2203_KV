mod models;
mod server;
mod utils;

#[tokio::main]
async fn main() {
    server::main();
}
