/// Window-level settings shared by all rendering backends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiWindowConfig {
    title: String,
    width: u32,
    height: u32,
}

impl UiWindowConfig {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: 1280,
            height: 820,
        }
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}
