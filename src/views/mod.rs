pub mod treemap;

use ratatui::Frame;
use crate::{app::App, mouse::HitBox};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Treemap,
    Heatmap,
    Minimap,
    River,
    Focus,
}

impl ViewMode {
    pub const ALL: [ViewMode; 5] = [
        ViewMode::Treemap,
        ViewMode::Heatmap,
        ViewMode::Minimap,
        ViewMode::River,
        ViewMode::Focus,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ViewMode::Treemap => "Treemap",
            ViewMode::Heatmap => "Heatmap",
            ViewMode::Minimap => "Minimap",
            ViewMode::River => "River",
            ViewMode::Focus => "Focus",
        }
    }

    pub fn index(self) -> usize {
        match self {
            ViewMode::Treemap => 0,
            ViewMode::Heatmap => 1,
            ViewMode::Minimap => 2,
            ViewMode::River => 3,
            ViewMode::Focus => 4,
        }
    }

    pub fn from_index(i: usize) -> Self {
        Self::ALL.get(i).copied().unwrap_or_default()
    }

    pub fn next(self) -> Self {
        Self::from_index((self.index() + 1) % Self::ALL.len())
    }

    pub fn prev(self) -> Self {
        let i = self.index();
        Self::from_index(if i == 0 { Self::ALL.len() - 1 } else { i - 1 })
    }
}

pub struct RenderResult {
    pub hit_boxes: Vec<HitBox>,
}

impl RenderResult {
    pub fn new() -> Self { Self { hit_boxes: Vec::new() } }
}

impl Default for RenderResult {
    fn default() -> Self { Self::new() }
}

pub fn draw(f: &mut Frame, app: &App) -> RenderResult {
    treemap::draw(f, app)
}