use ratatui::layout::Rect;

/// Identifies a clickable element.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum HitTarget {
    #[default]
    None,
    File(String),
    ViewTab(usize),
    SeismicCell(usize),
}

/// A rectangular region that can be clicked.
#[derive(Clone, Debug)]
pub struct HitBox {
    pub rect: Rect,
    pub target: HitTarget,
}

impl HitBox {
    pub fn new(rect: Rect, target: HitTarget) -> Self {
        Self { rect, target }
    }

    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.rect.x
            && x < self.rect.x + self.rect.width
            && y >= self.rect.y
            && y < self.rect.y + self.rect.height
    }
}

/// Current mouse state.
#[derive(Default)]
pub struct MouseState {
    pub x: u16,
    pub y: u16,
    pub hover: HitTarget,
}

impl MouseState {
    pub fn update_position(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    pub fn update_hover(&mut self, target: HitTarget) {
        self.hover = target;
    }
}

/// Find which hit box (if any) contains the given coordinates.
pub fn hit_test(x: u16, y: u16, boxes: &[HitBox]) -> HitTarget {
    boxes
        .iter()
        .find(|hb| hb.contains(x, y))
        .map(|hb| hb.target.clone())
        .unwrap_or_default()
}
