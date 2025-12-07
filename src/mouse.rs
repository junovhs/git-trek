use ratatui::layout::Rect;

#[derive(Clone, Debug)]
pub struct HitBox {
    pub rect: Rect,
    pub id: HitId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HitId {
    File(String),
    Commit(usize),
    TimelinePoint(usize),
    ViewTab(usize),
    None,
}

impl Default for HitId {
    fn default() -> Self { Self::None }
}

#[derive(Default)]
pub struct MouseState {
    pub x: u16,
    pub y: u16,
    pub hover: HitId,
}

impl MouseState {
    pub fn update_position(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    pub fn set_hover(&mut self, id: HitId) {
        self.hover = id;
    }
}

pub fn hit_test(x: u16, y: u16, boxes: &[HitBox]) -> HitId {
    for hb in boxes {
        if x >= hb.rect.x
            && x < hb.rect.x + hb.rect.width
            && y >= hb.rect.y
            && y < hb.rect.y + hb.rect.height
        {
            return hb.id.clone();
        }
    }
    HitId::None
}