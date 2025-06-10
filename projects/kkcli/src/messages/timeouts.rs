use super::Message;
use core::time::Duration;

const TIMEOUT: Duration = Duration::from_millis(5000);
const LONG_TIMEOUT: Duration = Duration::from_millis(5 * 60 * 1000);

impl Message {
    pub fn read_timeout(&self) -> Duration {
        match self {
            Message::ButtonAck(_) => LONG_TIMEOUT,
            _ => TIMEOUT,
        }
    }

    pub fn write_timeout(&self) -> Duration {
        match self {
            Message::FirmwareUpload(_) => LONG_TIMEOUT,
            _ => TIMEOUT,
        }
    }
}
