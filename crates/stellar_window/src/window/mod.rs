use fent_derive::Component;

#[derive(Component, Clone)]
pub struct Window {
    pub(crate) title: String,
    pub(crate) inner_size: WindowSize,
    pub(crate) resizable: bool,
    pub(crate) vulkan: bool,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            title: "Stellar Window".to_string(),
            inner_size: (1280, 720).into(),
            resizable: true,
            vulkan: true,
        }
    }
}

impl Window {
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }
}

#[derive(Clone, Copy)]
pub struct WindowSize {
    width: u32,
    height: u32,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
        }
    }
}

impl From<(u32, u32)> for WindowSize {
    fn from(value: (u32, u32)) -> Self {
        WindowSize {
            width: value.0,
            height: value.1,
        }
    }
}

impl WindowSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }
}
