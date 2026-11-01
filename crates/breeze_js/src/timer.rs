use boa_engine::JsValue;
use std::time::{Duration, Instant};

pub struct TimerEntry {
    pub id: u32,
    pub callback: JsValue,
    pub trigger_time: Instant,
    pub interval: Option<Duration>,
}

#[derive(Default)]
pub struct TimerQueue {
    next_id: u32,
    timers: Vec<TimerEntry>,
}

impl TimerQueue {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            timers: Vec::new(),
        }
    }

    pub fn set_timeout(&mut self, callback: JsValue, delay_ms: u64) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.timers.push(TimerEntry {
            id,
            callback,
            trigger_time: Instant::now() + Duration::from_millis(delay_ms),
            interval: None,
        });
        id
    }

    pub fn set_interval(&mut self, callback: JsValue, delay_ms: u64) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let d = Duration::from_millis(delay_ms.max(1));
        self.timers.push(TimerEntry {
            id,
            callback,
            trigger_time: Instant::now() + d,
            interval: Some(d),
        });
        id
    }

    pub fn clear(&mut self, id: u32) {
        self.timers.retain(|t| t.id != id);
    }

    pub fn collect_ready(&mut self) -> Vec<(u32, JsValue)> {
        let now = Instant::now();
        let mut ready = Vec::new();
        for t in &mut self.timers {
            if now >= t.trigger_time {
                ready.push((t.id, t.callback.clone()));
                if let Some(interval) = t.interval {
                    t.trigger_time = now + interval;
                }
            }
        }
        self.timers.retain(|t| t.interval.is_some() || now < t.trigger_time);
        ready
    }
}
