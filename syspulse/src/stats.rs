use std::collections::VecDeque;

#[derive(Clone)] // اضافه کردن Clone
pub struct Stats {
    cpu_history: VecDeque<f32>,
}

#[allow(dead_code)]
impl Stats {
    pub fn new(capacity: usize) -> Self {
        Stats {
            cpu_history: VecDeque::with_capacity(capacity),
        }
    }

    pub fn update(&mut self, cpu_usage: f32) {
        self.cpu_history.push_back(cpu_usage);
        if self.cpu_history.len() > self.cpu_history.capacity() {
            self.cpu_history.pop_front();
        }
    }

    pub fn average_cpu(&self) -> Option<f32> {
        if self.cpu_history.is_empty() {
            None
        } else {
            Some(self.cpu_history.iter().sum::<f32>() / self.cpu_history.len() as f32)
        }
    }

    pub fn cpu_trend(&self) -> Option<f32> {
        if self.cpu_history.len() > 10 {
            let recent_avg = self.cpu_history.iter().rev().take(5).sum::<f32>() / 5.0;
            Some(recent_avg - self.cpu_history[0])
        } else {
            None
        }
    }

    // متد جدید برای دسترسی به cpu_history
    pub fn cpu_history(&self) -> &VecDeque<f32> {
        &self.cpu_history
    }
}