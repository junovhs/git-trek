use ratatui::style::{Color, Style};

const CLR_TREMOR: Color = Color::Rgb(60, 80, 60);
const CLR_QUAKE: Color = Color::Rgb(120, 140, 60);
const CLR_MAJOR: Color = Color::Rgb(200, 160, 40);
const CLR_DISASTER: Color = Color::Rgb(220, 60, 40);
const CLR_QUIET: Color = Color::Rgb(30, 32, 35);
const CLR_DELETED: Color = Color::Rgb(80, 40, 40);
const CLR_NEW: Color = Color::Rgb(40, 80, 100);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intensity {
    None,
    Quiet,
    Tremor,
    Quake,
    Major,
    Disaster,
    Created,
    Deleted,
}

impl Intensity {
    pub fn calc(history: &crate::data::History, path: &str, commit_idx: usize) -> Self {
        let Some(file_hist) = history.files.get(path) else {
            return Self::None;
        };

        let current_lines = file_hist.lines_at(commit_idx);
        let prev_lines = commit_idx
            .checked_sub(1)
            .and_then(|i| file_hist.lines_at(i));

        match (prev_lines, current_lines) {
            (None, None) => Self::None,
            (None, Some(_)) => Self::Created,
            (Some(_), None) => Self::Deleted,
            (Some(old), Some(new)) => {
                let diff = old.abs_diff(new);
                if diff == 0 {
                    Self::Quiet
                } else if diff <= 10 {
                    Self::Tremor
                } else if diff <= 50 {
                    Self::Quake
                } else if diff <= 200 {
                    Self::Major
                } else {
                    Self::Disaster
                }
            }
        }
    }
}

pub fn format_cell(intensity: Intensity, is_current: bool) -> (String, Style) {
    let (char, bg) = match intensity {
        Intensity::None => (' ', CLR_QUIET),
        Intensity::Quiet => ('·', CLR_QUIET),
        Intensity::Tremor => ('░', CLR_TREMOR),
        Intensity::Quake => ('▒', CLR_QUAKE),
        Intensity::Major => ('▓', CLR_MAJOR),
        Intensity::Disaster => ('█', CLR_DISASTER),
        Intensity::Created => ('+', CLR_NEW),
        Intensity::Deleted => ('╳', CLR_DELETED),
    };

    let style = if is_current {
        Style::default().fg(Color::White).bg(bg)
    } else {
        Style::default().fg(bg)
    };

    (char.to_string(), style)
}
