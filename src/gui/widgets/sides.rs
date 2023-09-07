use iced_core::BorderRadius;

/// Represents the four sides of border
pub struct Sides<T> {
    /// top/topleft
    pub top: T,
    /// right/topmright
    pub right: T,
    /// bottom/bottomright
    pub bottom: T,
    /// left/bottomleft
    pub left: T,
}

impl<T> Sides<T> {
    pub fn top_left(&self) -> &T {
        &self.top
    }

    pub fn top_right(&self) -> &T {
        &self.right
    }

    pub fn bottom_right(&self) -> &T {
        &self.bottom
    }

    pub fn bottom_left(&self) -> &T {
        &self.left
    }
}

impl<T> From<[T; 4]> for Sides<T>
where
    T: Clone,
{
    fn from(value: [T; 4]) -> Self {
        Self {
            top: value[0].clone(),
            right: value[1].clone(),
            bottom: value[2].clone(),
            left: value[3].clone(),
        }
    }
}

impl From<[u32; 4]> for Sides<f32> {
    fn from(value: [u32; 4]) -> Self {
        Self {
            top: value[0] as f32,
            right: value[1] as f32,
            bottom: value[2] as f32,
            left: value[3] as f32,
        }
    }
}

impl<T> From<[T; 2]> for Sides<T>
where
    T: Clone,
{
    fn from(value: [T; 2]) -> Self {
        [
            value[0].clone(),
            value[1].clone(),
            value[0].clone(),
            value[1].clone(),
        ]
        .into()
    }
}

impl From<[u32; 2]> for Sides<f32> {
    fn from(value: [u32; 2]) -> Self {
        [
            value[0] as f32,
            value[1] as f32,
            value[0] as f32,
            value[1] as f32,
        ]
        .into()
    }
}

impl<T> From<T> for Sides<T>
where
    T: Clone,
{
    fn from(value: T) -> Self {
        [value.clone(), value].into()
    }
}

impl From<u32> for Sides<f32> {
    fn from(value: u32) -> Self {
        [value, value].into()
    }
}

impl<T> Clone for Sides<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            left: self.left.clone(),
            top: self.top.clone(),
            right: self.right.clone(),
            bottom: self.bottom.clone(),
        }
    }
}

impl<T> Copy for Sides<T> where T: Copy {}

impl Into<BorderRadius> for Sides<f32> {
    fn into(self) -> BorderRadius {
        [self.left, self.top, self.right, self.bottom].into()
    }
}
