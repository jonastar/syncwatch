use serde::Serialize;

#[derive(Serialize, Clone, Copy)]
pub enum State {
    Playing,
    Paused,
}

#[derive(Serialize)]
pub struct UpdateEvent {
    pub ts_millis: u64,
    pub state: State,
    pub media_url: String,
}
