use fent_derive::Component;

#[derive(Component, Clone)]
pub struct Window {
    pub(crate) title: String,
    pub(crate) inner_size: (u32, u32),
    pub(crate) resizable: bool,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            title: "Stellar Window".to_string(),
            inner_size: (680, 680),
            resizable: true,
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
    physical_width: u32,
    physical_height: u32,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            physical_width: 1280,
            physical_height: 720,
        }
    }
}

impl WindowSize {
    pub fn new(physical_width: u32, physical_height: u32) -> Self {
        Self {
            physical_width,
            physical_height,
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.physical_width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.physical_height
    }
}
