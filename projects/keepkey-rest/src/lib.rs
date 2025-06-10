pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

mod device;

use axum::{Router, routing::get, Extension};
use std::sync::Arc;
pub use device::{DeviceComm, MockDeviceComm};

pub fn create_router<D: DeviceComm>(device: D) -> Router {
    let device: Arc<dyn DeviceComm> = Arc::new(device);
    Router::new()
        .route("/api/v1/features", get(get_features))
        .layer(Extension(device))
}

async fn get_features(
    Extension(device): Extension<Arc<dyn DeviceComm>>,
) -> String {
    device.get_features().unwrap_or_else(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
