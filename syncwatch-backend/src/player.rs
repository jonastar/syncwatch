use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use tokio::sync::watch;

use crate::events::State;

pub type PlayerStateHandle = Arc<RwLock<PlayerState>>;

pub struct PlayerState {
    current_state: State,
    unpaused_at: Instant,
    media_url: String,
    start_offset: Duration,
    evt_send: watch::Sender<LastPlayerState>,
}

impl PlayerState {
    pub fn new() -> Self {
        let (sender, _) = watch::channel(LastPlayerState {
            ts: Duration::ZERO,
            state: State::Paused,
            media_url: String::new(),
        });
        Self {
            current_state: State::Paused,
            unpaused_at: Instant::now(),
            media_url: String::new(),
            start_offset: Duration::ZERO,
            evt_send: sender,
        }
    }

    fn send_status_update(&self) {
        self.evt_send.send_replace(LastPlayerState {
            ts: self.get_current_timestamp(),
            state: self.current_state,
            media_url: self.media_url.clone(),
        });
    }

    pub fn get_current_timestamp(&self) -> Duration {
        match self.current_state {
            State::Playing => {
                let dur_since_unpaused = self.unpaused_at.elapsed();
                self.start_offset + dur_since_unpaused
            }
            State::Paused => self.start_offset,
        }
    }

    pub fn pause(&mut self) {
        if matches!(self.current_state, State::Paused) {
            // already paused
            return;
        }

        self.current_state = State::Paused;

        // adjust the next start offset accordingly
        self.start_offset += self.unpaused_at.elapsed();

        self.send_status_update();
    }

    pub fn unpause(&mut self) {
        if matches!(self.current_state, State::Playing) {
            // already paused
            return;
        }

        self.current_state = State::Playing;
        self.unpaused_at = Instant::now();

        self.send_status_update();
    }

    pub fn seek(&mut self, new_ts: Duration) {
        self.unpaused_at = Instant::now();
        self.start_offset = new_ts;
        self.send_status_update();
    }

    pub fn change_media(&mut self, new_url: String) {
        self.current_state = State::Paused;
        self.media_url = new_url;
        self.start_offset = Duration::ZERO;
        self.send_status_update();
    }

    pub fn current_state(&self) -> LastPlayerState {
        LastPlayerState {
            ts: self.get_current_timestamp(),
            state: self.current_state,
            media_url: self.media_url.clone(),
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<LastPlayerState> {
        self.evt_send.subscribe()
    }
}

#[derive(Clone)]
pub struct LastPlayerState {
    pub ts: Duration,
    pub state: State,
    pub media_url: String,
}

// impl LastPlayerState {}
