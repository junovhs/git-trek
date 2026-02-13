mod terrain;

use ratatui::Frame;

use crate::app::App;
use crate::mouse::HitBox;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Terrain,
    Seismic,
    Strata,
    Flow,
    Constellation,
    Surgery,
}

impl ViewMode {
    pub const ALL: [ViewMode; 6] = [
        ViewMode::Terrain,
        ViewMode::Seismic,
        ViewMode::Strata,
        ViewMode::Flow,
        ViewMode::Constellation,
        ViewMode::Surgery,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ViewMode::Terrain => "Terrain",
            ViewMode::Seismic => "Seismic",
            ViewMode::Strata => "Strata",
            ViewMode::Flow => "Flow",
            ViewMode::Constellation => "Stars",
            ViewMode::Surgery => "Surgery",
        }
    }

    pub fn index(self) -> usize {
        match self {
            ViewMode::Terrain => 0,
            ViewMode::Seismic => 1,
            ViewMode::Strata => 2,
            ViewMode::Flow => 3,
            ViewMode::Constellation => 4,
            ViewMode::Surgery => 5,
        }
    }

    pub fn from_index(i: usize) -> Self {
        Self::ALL.get(i).copied().unwrap_or_default()
    }

    #[must_use]
    pub fn next(self) -> Self {
        Self::from_index((self.index() + 1) % Self::ALL.len())
    }

    #[must_use]
    pub fn prev(self) -> Self {
        let i = self.index();
        Self::from_index(if i == 0 { Self::ALL.len() - 1 } else { i - 1 })
    }
}

/// Result of rendering a view, containing hit boxes for mouse interaction.
pub struct Render {
    pub hit_boxes: Vec<HitBox>,
}

impl Render {
    pub fn new() -> Self {
        Self {
            hit_boxes: Vec::new(),
        }
    }
}

impl Default for Render {
    fn default() -> Self {
        Self::new()
    }
}

/// Draw the current view.
pub fn draw(f: &mut Frame, app: &App) -> Render {
    terrain::draw(f, app)
}
