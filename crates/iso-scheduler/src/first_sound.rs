//! A bounded first-sound clock, not a production clock-recovery/resampling loop.
use std::time::{Duration, Instant};

pub const SDU_PERIOD: Duration = Duration::from_millis(10);
pub const MAX_LATENESS: Duration = Duration::from_millis(30);

pub struct StereoPacer { epoch: Instant }
impl StereoPacer {
    pub fn new(epoch: Instant) -> Self { Self { epoch } }
    pub fn deadline(&self, frame: u32) -> Instant { self.epoch + SDU_PERIOD * frame }
    pub fn lateness(&self, frame: u32, now: Instant) -> Result<Duration, String> {
        let late = now.saturating_duration_since(self.deadline(frame));
        if late > MAX_LATENESS {
            return Err(format!("ISO pacing deadline missed by {}us at frame {frame}; refusing a catch-up burst", late.as_micros()));
        }
        Ok(late)
    }
}

/// Reserve room for BOTH ears before queueing either. `pending` includes queued
/// and in-flight HCI packets; max_in_flight is the reported controller window.
pub fn pair_fits(pending: usize, max_in_flight: usize) -> bool {
    max_in_flight >= 2 && pending <= max_in_flight - 2
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deadlines_do_not_drift_with_actual_wakeup_times() {
        let now=Instant::now(); let p=StereoPacer::new(now);
        assert_eq!(p.deadline(100).duration_since(now), Duration::from_secs(1));
        assert_eq!(p.lateness(1, now + Duration::from_millis(15)).unwrap(), Duration::from_millis(5));
        assert!(p.lateness(1, now + Duration::from_millis(41)).is_err());
    }
    #[test]
    fn never_spend_the_last_single_credit_on_one_ear_only() {
        assert!(pair_fits(6,8)); assert!(!pair_fits(7,8));
        assert!(!pair_fits(0,0)); assert!(!pair_fits(0,1));
        assert!(!pair_fits(usize::MAX,8));
    }
}
