use {
    super::constants::REPLAY_WINDOW,
    serde::{Deserialize, Serialize},
};

/// Receivers dedupe by `seq` and accept gaps (MSG-06). Sends are independent HTTP requests and can
/// arrive out of order, so this keeps the highest seen plus the seen values within the window.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplayWindow {
    pub max: u64,
    pub seen: Vec<u64>,
}

impl ReplayWindow {
    pub fn accept(&mut self, seq: u64) -> bool {
        if seq == 0
            || seq + REPLAY_WINDOW <= self.max
            || self.seen.contains(&seq)
        {
            return false;
        }
        self.seen.push(seq);
        self.max = self.max.max(seq);
        let floor = self.max.saturating_sub(REPLAY_WINDOW);
        self.seen.retain(|s| *s > floor);
        true
    }
}
