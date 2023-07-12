use std::{fmt, time::Duration};

#[derive(Clone, Copy)]
pub struct Time {
    minutes: u32,
    seconds: u32,
}

impl From<u32> for Time {
    fn from(seconds: u32) -> Self {
        Self {
            seconds: seconds % 60,
            minutes: seconds / 60,
        }
    }
}

impl From<Duration> for Time {
    fn from(duration: Duration) -> Self {
        assert!(
            duration < Duration::from_secs(60 * 60),
            "Duration is bigger than an hour"
        );

        Self::from(duration.as_secs() as u32)
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { minutes, seconds } = self;
        write!(f, "{minutes:02}:{seconds:02}")
    }
}
