use salvo::prelude::*;

const ADDRESS: &str = "127.0.0.1:8080";
const STATIC: &str = "site";

#[tokio::main]
async fn main() {
    let router = Router::with_path("<**path>").get(
        StaticDir::new([STATIC])
            .defaults("index.html")
            .fallback("index.html")
            .listing(true),
    );
    let listener = TcpListener::new(ADDRESS).bind().await;
    let server = Server::new(listener);

    println!("CHIP-8 Emulator at http://{}", ADDRESS);
    server.serve(router).await;
}
