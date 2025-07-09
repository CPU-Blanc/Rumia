use rumia::server;

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server().launch().await?;
    Ok(())
}
