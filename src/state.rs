use google_drive3::DriveHub;
use tokio::sync::Mutex;

pub struct AppState {
    pub hub: Mutex<DriveHub>,
}

