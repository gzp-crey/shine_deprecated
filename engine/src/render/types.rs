#![deny(missing_docs)]
#![deny(missing_copy_implementations)]


/// Structure to store the window size.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Size {
    /// The width.
    pub width: u32,
    /// The height.
    pub height: u32,
}

impl From<[u32; 2]> for Size {
    #[inline(always)]
    fn from(value: [u32; 2]) -> Size {
        Size {
            width: value[0],
            height: value[1],
        }
    }
}

impl From<(u32, u32)> for Size {
    #[inline(always)]
    fn from(value: (u32, u32)) -> Size {
        Size {
            width: value.0,
            height: value.1,
        }
    }
}

impl From<Size> for [u32; 2] {
    #[inline(always)]
    fn from(value: Size) -> [u32; 2] {
        [value.width, value.height]
    }
}

impl From<Size> for (u32, u32) {
    #[inline(always)]
    fn from(value: Size) -> (u32, u32) {
        (value.width, value.height)
    }
}


/// Structure to store the window position.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Position {
    /// The x coordinate.
    pub x: i32,
    /// The y coordinate.
    pub y: i32,
}

impl From<[i32; 2]> for Position {
    #[inline(always)]
    fn from(value: [i32; 2]) -> Position {
        Position {
            x: value[0],
            y: value[1],
        }
    }
}

impl From<(i32, i32)> for Position {
    #[inline(always)]
    fn from(value: (i32, i32)) -> Position {
        Position {
            x: value.0,
            y: value.1,
        }
    }
}

impl From<Position> for [i32; 2] {
    #[inline(always)]
    fn from(value: Position) -> [i32; 2] {
        [value.x, value.y]
    }
}

impl From<Position> for (i32, i32) {
    #[inline(always)]
    fn from(value: Position) -> (i32, i32) {
        (value.x, value.y)
    }
}


/// Float array with 4 components
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Float32x4(pub f32, pub f32, pub f32, pub f32);

impl Default for Float32x4 {
    #[inline(always)]
    fn default() -> Float32x4 {
        Float32x4(0., 0., 0., 0.)
    }
}

impl From<[f32; 4]> for Float32x4 {
    #[inline(always)]
    fn from(value: [f32; 4]) -> Float32x4 {
        Float32x4(value[0], value[1], value[2], value[3])
    }
}

impl From<(f32, f32, f32, f32)> for Float32x4 {
    #[inline(always)]
    fn from(value: (f32, f32, f32, f32)) -> Float32x4 {
        Float32x4(value.0, value.1, value.2, value.3)
    }
}

/// Constructs Float32x4 from multiple expressions
/// # Example
/// assert!( f32x4!(1, 2, 3, 4) == Float32x4(1., 2., 3., 4.) );
/// assert!( f32x4!(1) == Float32x4(1., 1., 1, 1.) );
/// assert!( f32x4!() == Float32x4(0., 0., 0., 0.) );
#[macro_export]
macro_rules! f32x4 {
    ($_x:expr, $_y:expr, $_z:expr, $_w:expr) => { $crate::render::Float32x4($_x as f32, $_y as f32, $_z as f32, $_w as f32) };
    ($_x:expr) => { $crate::render::Float32x4($_x as f32, $_x as f32, $_x as f32, $_x as f32) };
    () => { {let f : $crate::render::Float32x4 = Default::default(); f} };
}


/// Float array with 3 components
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Float32x3(pub f32, pub f32, pub f32);

impl Default for Float32x3 {
    fn default() -> Float32x3 {
        Float32x3(0., 0., 0.)
    }
}

impl From<[f32; 3]> for Float32x3 {
    #[inline(always)]
    fn from(value: [f32; 3]) -> Float32x3 {
        Float32x3(value[0], value[1], value[2])
    }
}

impl From<(f32, f32, f32)> for Float32x3 {
    #[inline(always)]
    fn from(value: (f32, f32, f32)) -> Float32x3 {
        Float32x3(value.0, value.1, value.2)
    }
}

/// Constructs Float32x3 from multiple expressions
///
/// # Example
/// assert!( f32x3!(1, 2, 3) == Float32x3(1., 2., 3.) );
/// assert!( f32x3!(1) == Float32x3(1., 1., 1) );
/// assert!( f32x3!() == Float32x3(0., 0., 0.) );
#[macro_export]
macro_rules! f32x3 {
    ($_x:expr, $_y:expr, $_z:expr) => { $crate::render::Float32x3($_x as f32, $_y as f32, $_z as f32) };
    ($_x:expr) => { $crate::render::Float32x3::new($_x as f32, $_x as f32, $_x as f32) };
    () => { {let f : $crate::render::Float32x3 = Default::default(); f} };
}


/// Float array with 2 components
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Float32x2(pub f32, pub f32);

impl Default for Float32x2 {
    fn default() -> Float32x2 {
        Float32x2(0., 0.)
    }
}

impl From<[f32; 2]> for Float32x2 {
    #[inline(always)]
    fn from(value: [f32; 2]) -> Float32x2 {
        Float32x2(value[0], value[1])
    }
}

impl From<(f32, f32)> for Float32x2 {
    #[inline(always)]
    fn from(value: (f32, f32)) -> Float32x2 {
        Float32x2(value.0, value.1)
    }
}

/// Constructs Float32x3 from multiple expressions
///
/// # Example
/// assert!( f32x2!(1, 2) == Float32x2(1., 2.) );
/// assert!( f32x2!(1) == Float32x2(1., 1.) );
/// assert!( f32x2!() == Float32x2(0., 0.) );
#[macro_export]
macro_rules! f32x2 {
    ($_x:expr, $_y:expr) => { $crate::render::Float32x2($_x as f32, $_y as f32) };
    ($_x:expr) => { $crate::render::Float32x2::new($_x as f32, $_x as f32) };
    () => { {let f : $crate::render::Float32x2 = Default::default(); f} };
}

