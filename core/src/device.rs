#[derive(Default)]
pub struct Device {
    pub is_portrait: bool,
    pub is_mobile: bool,
}

impl Device {
    pub fn is_portrait(&self) -> bool {
        self.is_portrait
    }

    pub fn is_mobile(&self) -> bool {
        self.is_mobile
    }
}
