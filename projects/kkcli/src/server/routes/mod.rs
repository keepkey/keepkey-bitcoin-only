// Route modules
pub mod common;
pub mod auth;
pub mod device;
pub mod system;
pub mod system_management;
pub mod addresses;
pub mod bitcoin;
pub mod debug;
pub mod manufacturing;
pub mod raw;
pub mod websocket;



// Re-export all types and handlers for convenience
pub use common::*;
pub use auth::*;
pub use device::*;
pub use system::*;
pub use system_management::*;
pub use addresses::*;
pub use bitcoin::*;
pub use debug::*;
pub use manufacturing::*;
pub use raw::*;
pub use websocket::*;

 