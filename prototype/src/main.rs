#[tokio::main]
async fn main() {
    prototype::run_localhost()
        .await
        .expect("prototype demo failed");
}
