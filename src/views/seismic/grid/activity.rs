use crate::data::History;

pub fn sort_by_activity(
    history: &History,
    files: &[String],
    start: usize,
    end: usize,
) -> Vec<String> {
    let mut scored: Vec<_> = files
        .iter()
        .map(|path| {
            let score = calc_activity_score(history, path, start, end);
            (path.clone(), score)
        })
        .collect();

    scored.sort_by(|a, b| match b.1.cmp(&a.1) {
        std::cmp::Ordering::Equal => a.0.cmp(&b.0),
        other => other,
    });
    scored.into_iter().map(|(path, _)| path).collect()
}

pub fn has_activity(history: &History, path: &str, start: usize, end: usize) -> bool {
    let Some(file_hist) = history.files.get(path) else {
        return false;
    };

    for i in start..end {
        let prev_lines = if i == 0 {
            None
        } else {
            file_hist.lines_at(i - 1)
        };
        let curr_lines = file_hist.lines_at(i);

        match (prev_lines, curr_lines) {
            (None, Some(_)) | (Some(_), None) => return true,
            (Some(prev), Some(curr)) if prev != curr => return true,
            _ => {}
        }
    }
    false
}

fn calc_activity_score(history: &History, path: &str, start: usize, end: usize) -> u64 {
    let Some(file_hist) = history.files.get(path) else {
        return 0;
    };

    let mut score = 0u64;
    for i in start..end {
        let prev_lines = if i == 0 {
            None
        } else {
            file_hist.lines_at(i - 1)
        };
        let curr_lines = file_hist.lines_at(i);

        if let (Some(prev), Some(curr)) = (prev_lines, curr_lines) {
            score += prev.abs_diff(curr) as u64;
        } else if prev_lines.is_some() || curr_lines.is_some() {
            score += 10;
        }
    }
    score
}
