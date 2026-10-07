#[tokio::main]
async fn main() {
    if let Err(err) = prototype::run_localhost().await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
