use keepkey_rest::{create_router, MockDeviceComm};
use axum::Server;

#[tokio::main]
async fn main() {
    // Use a mock device for demonstration; replace with real device logic as needed
    let app = create_router(MockDeviceComm::default());
    let addr = "127.0.0.1:8080".parse().unwrap();
    println!("Serving REST API on http://{}", addr);
    Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}
