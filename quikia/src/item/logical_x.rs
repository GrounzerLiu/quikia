use std::ops::{Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use crate::item::LayoutDirection;

#[derive(Clone, Copy, Debug)]
pub struct LogicalX {
    direction: LayoutDirection,
    width: f32,
    start_x: f32,
    logical_x: f32
}

impl LogicalX {
    pub fn new(direction: LayoutDirection, start_x: f32, logical_x: f32, width: f32) -> Self {
        Self {
            direction,
            width,
            start_x,
            logical_x,
        }
    }

    pub fn direction(&self) -> LayoutDirection {
        self.direction
    }

    pub fn physical_value(&self, width: f32) -> f32 {
        match self.direction {
            LayoutDirection::LeftToRight => self.logical_x,
            LayoutDirection::RightToLeft => self.start_x + self.width - (self.logical_x - self.start_x) - width,
        }
    }

    pub fn logical_value(&self) -> f32 {
        self.logical_x
    }
}

impl Add<f32> for LogicalX {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        let logical_x = self.logical_x + rhs;
        Self {
            logical_x,
            ..self
        }
    }
}

impl Sub<f32> for LogicalX {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        let logical_x = self.logical_x - rhs;
        Self {
            logical_x,
            ..self
        }
    }
}

impl Mul<f32> for LogicalX {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        let logical_x = self.logical_x * rhs;
        Self {
            logical_x,
            ..self
        }
    }
}

impl Div<f32> for LogicalX {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        let logical_x = self.logical_x / rhs;
        Self {
            logical_x,
            ..self
        }
    }
}

impl Add<LogicalX> for LogicalX {
    type Output = Self;

    fn add(self, rhs: LogicalX) -> Self::Output {
        if self.direction != rhs.direction || self.start_x != rhs.start_x || self.width != rhs.width {
            panic!("LogicalX can't add LogicalX with different direction or start_x or width");
        }
        let logical_x = self.logical_x + rhs.logical_x;
        Self {
            logical_x,
            ..self
        }
    }
}

impl Sub<LogicalX> for LogicalX {
    type Output = Self;

    fn sub(self, rhs: LogicalX) -> Self::Output {
        if self.direction != rhs.direction || self.start_x != rhs.start_x || self.width != rhs.width {
            panic!("LogicalX can't sub LogicalX with different direction or start_x or width");
        }
        let logical_x = self.logical_x - rhs.logical_x;
        Self {
            logical_x,
            ..self
        }
    }
}

impl Mul<LogicalX> for LogicalX {
    type Output = Self;

    fn mul(self, rhs: LogicalX) -> Self::Output {
        if self.direction != rhs.direction || self.start_x != rhs.start_x || self.width != rhs.width {
            panic!("LogicalX can't mul LogicalX with different direction or start_x or width");
        }
        let logical_x = self.logical_x * rhs.logical_x;
        Self {
            logical_x,
            ..self
        }
    }
}

impl Div<LogicalX> for LogicalX {
    type Output = Self;

    fn div(self, rhs: LogicalX) -> Self::Output {
        if self.direction != rhs.direction || self.start_x != rhs.start_x || self.width != rhs.width {
            panic!("LogicalX can't div LogicalX with different direction or start_x or width");
        }
        let logical_x = self.logical_x / rhs.logical_x;
        Self {
            logical_x,
            ..self
        }
    }
}

impl AddAssign<f32> for LogicalX {
    fn add_assign(&mut self, rhs: f32) {
        self.logical_x += rhs;
    }
}

impl SubAssign<f32> for LogicalX {
    fn sub_assign(&mut self, rhs: f32) {
        self.logical_x -= rhs;
    }
}

impl MulAssign<f32> for LogicalX {
    fn mul_assign(&mut self, rhs: f32) {
        self.logical_x *= rhs;
    }
}

impl DivAssign<f32> for LogicalX {
    fn div_assign(&mut self, rhs: f32) {
        self.logical_x /= rhs;
    }
}

impl Deref for LogicalX {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.logical_x
    }
}

impl DerefMut for LogicalX {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.logical_x
    }
}





