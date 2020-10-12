#[derive(Copy, Clone, Default, PartialOrd, PartialEq, Debug)]
pub struct Time(f64);

impl Time {
    pub fn from_millis(millis: f64) -> Self {
        Time(millis)
    }
}

impl ::std::ops::Add<Duration> for Time {
    type Output = Time;
    fn add(self, d: Duration) -> Time {
        Time(self.0 - d.0)
    }
}

impl ::std::ops::Sub<Time> for Time {
    type Output = Duration;
    fn sub(self, other: Time) -> Duration {
        Duration(self.0 - other.0)
    }
}

impl ::std::ops::Rem<Duration> for Time {
    type Output = Duration;
    fn rem(self, divisor: Duration) -> Duration {
        Duration(self.0 % divisor.0)
    }
}

#[derive(Copy, Clone, Default, PartialOrd, PartialEq, Debug)]
pub struct Duration(f64);

impl Duration {
    pub fn from_millis(millis: f64) -> Self {
        Duration(millis)
    }

    pub fn as_pi(&self, period: Duration) -> f64 {
        self.0 / period.0 * ::std::f64::consts::PI
    }
}

impl ::std::ops::Add<Duration> for Duration {
    type Output = Duration;
    fn add(self, other: Duration) -> Duration {
        Duration(self.0 + other.0)
    }
}

impl ::std::ops::Mul<Duration> for f64 {
    type Output = Duration;
    fn mul(self, d: Duration) -> Duration {
        Duration(self * d.0)
    }
}

impl ::std::ops::Mul<f64> for Duration {
    type Output = Duration;
    fn mul(self, d: f64) -> Duration {
        Duration(self.0 * d)
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
