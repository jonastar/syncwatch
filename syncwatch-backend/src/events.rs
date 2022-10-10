use serde::Serialize;

#[derive(Serialize)]
pub enum State {
    Playing,
    Paused,
}

#[derive(Serialize)]
#[serde(tag = "t")]
pub enum Event {
    Ready(ReadyEvent),
    ClockUpdate(u64),
    StateUpdate(State),
}

#[derive(Serialize)]
#[serde(tag = "t")]
pub struct ReadyEvent {
    clock: u64,
    state: State,
}
