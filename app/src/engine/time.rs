#[derive(Copy, Clone, Default, PartialOrd, PartialEq, Debug)]
pub struct Time(f64);

impl Time {
    pub fn from_millis(millis: f64) -> Self {
        Time(millis)
    }
}

impl ::std::ops::Sub for Time {
    type Output = Duration;
    fn sub(self, other: Time) -> Duration {
        Duration(self.0 - other.0)
    }
}

#[derive(Copy, Clone, Default, PartialOrd, PartialEq, Debug)]
pub struct Duration(f64);

impl Duration {
    pub fn from_millis(millis: f64) -> Self {
        Duration(millis)
    }
}

impl ::std::ops::Div<Duration> for usize {
    type Output = Rate;
    fn div(self, d: Duration) -> Rate {
        Rate(self as f64 * 1000.0 / d.0)
    }
}

#[derive(Copy, Clone, Default, PartialOrd, PartialEq, Debug)]
pub struct Rate(f64);

pub struct Framerate {
    buf: [Time; 32],
    index: usize,
}

impl Framerate {
    pub fn new() -> Self {
        Framerate {
            buf: [::std::default::Default::default(); 32],
            index: 0,
        }
    }

    pub fn record_timestamp(&mut self, t: Time) {
        self.buf[self.index] = t;
        self.index = (self.index + 1) % self.buf.len();
    }

    pub fn rate(&self) -> Rate {
        let len = self.buf.len();
        let first = self.buf[self.index];
        let last = self.buf[(self.index + (len - 1)) % len];
        len / (last - first)
    }
}
