use std::time::{Duration, Instant};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Widget};

pub struct AnimClip {
    frames: &'static [&'static str],  // frames byte-compiled as str
    frame_ms: u64,
    looping: bool,
}

impl AnimClip {
    pub fn frame_at(&self, elapsed: Duration) -> &'static str {
        let i = (elapsed.as_millis() as u64 / self.frame_ms) as usize;
        if self.looping {
            self.frames[i % self.frames.len()]
        } else {
            self.frames[i.min(self.frames.len() - 1)] // hold on last frame
        }
    }
}

const IDLE: AnimClip = AnimClip {
    frames: &[
        include_str!("../ascii/staz/idle/frame1.txt"),
        include_str!("../ascii/staz/idle/frame2.txt"),
        include_str!("../ascii/staz/idle/frame1.txt"),
        include_str!("../ascii/staz/idle/frame3.txt"),
        include_str!("../ascii/staz/idle/frame1.txt"),
        include_str!("../ascii/staz/idle/frame4.txt"),
        include_str!("../ascii/staz/idle/frame5.txt"),
        include_str!("../ascii/staz/idle/frame4.txt"),
    ],
    frame_ms: 600,
    looping: true,
};
const SLEEPING: AnimClip = AnimClip {
    frames: &[
        include_str!("../ascii/staz/sleeping/frame1.txt"),
        include_str!("../ascii/staz/sleeping/frame2.txt"),
        include_str!("../ascii/staz/sleeping/frame3.txt"),
        include_str!("../ascii/staz/sleeping/frame2.txt"),
        include_str!("../ascii/staz/sleeping/frame1.txt"),
    ],
    frame_ms: 1000,
    looping: true,
};
const WORKING: AnimClip = AnimClip {
    frames: &[
        include_str!("../ascii/staz/working/frame1.txt"),
        include_str!("../ascii/staz/working/frame2.txt"),
        include_str!("../ascii/staz/working/frame3.txt"),
        include_str!("../ascii/staz/working/frame4.txt"),
        include_str!("../ascii/staz/working/frame5.txt"),
    ],
    frame_ms: 400,
    looping: true,
};
const CELEBRATING: AnimClip = AnimClip {
    frames: &[
        include_str!("../ascii/staz/celebrating/frame1.txt"),
        include_str!("../ascii/staz/celebrating/frame2.txt"),
    ],
    frame_ms: 600,
    looping: true,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Idle,
    Sleeping,
    Working,
    Celebrating,
}

/// How long Staz celebrates before settling back to idle.
const CELEBRATE_FOR: Duration = Duration::from_secs(3);

pub struct Staz {
    mood: Mood,
    since: Instant,
}

impl Staz {
    pub fn new() -> Self {
        Self{ mood: Mood::Idle, since: Instant::now() }
    }

    pub fn set(&mut self, mood: Mood) {
        if mood != self.mood {
            self.mood = mood;
            self.since = Instant::now();
        }
    }

    pub fn tick(&mut self) {
        let t = self.since.elapsed();
        match self.mood {
            Mood::Idle if t > Duration::from_secs(60) => self.set(Mood::Sleeping),
            Mood::Celebrating if t > CELEBRATE_FOR => self.set(Mood::Idle),
            _ => {}
        }
    }

    fn clip(&self) -> &'static AnimClip {
        match self.mood {
            Mood::Idle => &IDLE,
            Mood::Sleeping => &SLEEPING,
            Mood::Working => &WORKING,
            Mood::Celebrating => &CELEBRATING,
        }
    }

    pub fn frame(&self) -> &'static str {
        self.clip().frame_at(self.since.elapsed())
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.frame()).render(area, buf);
    }
}

