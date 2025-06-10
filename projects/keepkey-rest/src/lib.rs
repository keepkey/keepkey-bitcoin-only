pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub mod device;
pub mod routes {
    pub mod bitcoin;
    pub mod v2;
}
pub use routes::bitcoin::bitcoin_router;
pub use routes::v2::v2_router;

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
) -> axum::Json<crate::device::Features> {
    match device.get_features() {
        Ok(features) => axum::Json(features),
        Err(e) => panic!("Failed to get features: {}", e),
    }
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
