mod youtubei;
mod youtube;
mod models;
mod errors;
mod api;

#[tokio::main]
async fn main() {
    let app = api::create_router();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server starting on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}