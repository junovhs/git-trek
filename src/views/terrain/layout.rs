use ratatui::layout::Rect;

/// Compute treemap layout for files in a given area.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn compute(files: &[(&str, usize)], area: Rect) -> Vec<(String, usize, Rect)> {
    let total: usize = files.iter().map(|(_, s)| *s).sum();
    if total == 0 || area.width < 4 || area.height < 2 {
        return vec![];
    }

    let mut result = Vec::new();
    let mut remaining = area;
    let max_files = 24;

    for (path, lines) in files.iter().take(max_files) {
        if remaining.width < 6 || remaining.height < 2 {
            break;
        }

        let ratio = (*lines as f64) / (total as f64);
        let horizontal = remaining.width >= remaining.height;

        let rect = if horizontal {
            let w = (f64::from(remaining.width) * ratio)
                .max(6.0)
                .min(f64::from(remaining.width)) as u16;
            let r = Rect::new(remaining.x, remaining.y, w, remaining.height);
            remaining.x += w;
            remaining.width = remaining.width.saturating_sub(w);
            r
        } else {
            let h = (f64::from(remaining.height) * ratio)
                .max(2.0)
                .min(f64::from(remaining.height)) as u16;
            let r = Rect::new(remaining.x, remaining.y, remaining.width, h);
            remaining.y += h;
            remaining.height = remaining.height.saturating_sub(h);
            r
        };

        result.push(((*path).to_string(), *lines, rect));
    }

    result
}
