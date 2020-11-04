use std::{
    fmt,
    ops,
    time::Duration
};

#[derive(Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Time {
    duration: Duration
}

impl From<Duration> for Time {
    fn from(duration: Duration) -> Self {
        Time { duration }
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let secs = self.duration.as_secs();
        let minutes = secs / 60;
        let secs = secs % 60;

        write!(f, "{}:{:02}", minutes, secs)
    }
}

impl ops::Div for Time {
    type Output = f32;

    fn div(self, rhs: Time) -> Self::Output {
        self.duration.as_secs_f32()
            / rhs.duration.as_secs_f32()
    }
}
