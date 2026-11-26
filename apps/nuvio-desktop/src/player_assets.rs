use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const FONT_REGULAR: &[u8] = include_bytes!("../ui/assets/jetbrains_sans_regular.ttf");
const FONT_SEMIBOLD: &[u8] = include_bytes!("../ui/assets/jetbrains_sans_semibold.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../ui/assets/jetbrains_sans_bold.ttf");

const CONTROLS_HTML: &str = include_str!("../ui/player-ui/controls.html");
const CONTROLS_CSS: &str = include_str!("../ui/player-ui/controls.css");
const CONTROLS_JS: &str = include_str!("../ui/player-ui/controls.js");
const LOADING_INDICATOR_JS: &str = include_str!("../ui/player-ui/loading-indicator.js");
const PARTY_CSS: &str = include_str!("../ui/player-ui/party.css");
const PARTY_JS: &str = include_str!("../ui/player-ui/party.js");

/// Exports embedded `player-ui` assets + fonts to `cache_dir/player-ui` and returns
/// the `file://...` URL to `controls.html`.
pub fn export_player_ui_assets(cache_dir: &Path) -> Result<String, String> {
    let dir = cache_dir.join("player-ui");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let font_reg = dir.join("jetbrains_sans_regular.ttf");
    let font_semi = dir.join("jetbrains_sans_semibold.ttf");
    let font_bold = dir.join("jetbrains_sans_bold.ttf");
    write_if_missing_or_diff(&font_reg, FONT_REGULAR)?;
    write_if_missing_or_diff(&font_semi, FONT_SEMIBOLD)?;
    write_if_missing_or_diff(&font_bold, FONT_BOLD)?;

    let font_faces = format!(
        "@font-face {{ font-family: 'NuvioSans'; src: url('{}') format('truetype'); font-weight: 400; font-style: normal; }}\n\
         @font-face {{ font-family: 'NuvioSans'; src: url('{}') format('truetype'); font-weight: 500 600; font-style: normal; }}\n\
         @font-face {{ font-family: 'NuvioSans'; src: url('{}') format('truetype'); font-weight: 700; font-style: normal; }}\n",
        path_to_file_url(&font_reg),
        path_to_file_url(&font_semi),
        path_to_file_url(&font_bold),
    );
    let rendered_css = CONTROLS_CSS.replace("/* __NUVIO_PLAYER_FONT_FACES__ */", &font_faces);

    write_if_missing_or_diff(&dir.join("controls.css"), rendered_css.as_bytes())?;
    write_if_missing_or_diff(&dir.join("controls.js"), CONTROLS_JS.as_bytes())?;
    write_if_missing_or_diff(&dir.join("loading-indicator.js"), LOADING_INDICATOR_JS.as_bytes())?;
    write_if_missing_or_diff(&dir.join("party.css"), PARTY_CSS.as_bytes())?;
    write_if_missing_or_diff(&dir.join("party.js"), PARTY_JS.as_bytes())?;

    write_if_missing_or_diff(&dir.join("seek-preview.js"), include_bytes!("../ui/player-ui/seek-preview.js"))?;

    let html_path = dir.join("controls.html");
    write_if_missing_or_diff(&html_path, CONTROLS_HTML.as_bytes())?;
    Ok(path_to_file_url(&html_path))
}

fn write_if_missing_or_diff(path: &PathBuf, bytes: &[u8]) -> Result<(), String> {
    if let Ok(existing) = fs::read(path)
        && existing == bytes
    {
        return Ok(());
    }
    fs::write(path, bytes).map_err(|e| e.to_string())
}

pub fn path_to_file_url(path: &Path) -> String {
    let raw = path.to_string_lossy().replace('\\', "/");
    if raw.starts_with('/') {
        format!("file://{raw}")
    } else {
        format!("file:///{raw}")
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleCueItem {
    pub index: usize,
    pub time_ms: i64,
    pub time_label: String,
    pub text: String,
}

/// Parses SRT, WebVTT, or ASS/SSA subtitle text into sorted cue entries for AutoSync.
pub fn parse_subtitle_cues(raw: &str) -> Vec<SubtitleCueItem> {
    let cleaned = raw.trim_start_matches('\u{feff}').replace("\r\n", "\n");
    let mut cues = Vec::new();

    if cleaned.contains("[Events]") && cleaned.contains("Dialogue:") {
        for line in cleaned.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("Dialogue:") {
                let parts: Vec<&str> = rest.splitn(10, ',').collect();
                if parts.len() == 10
                    && let Some(start_ms) = parse_timestamp_ms(parts[1].trim())
                {
                    let text = strip_tags_and_ass(parts[9].trim());
                    if !text.is_empty() {
                        cues.push((start_ms, text));
                    }
                }
            }
        }
    } else {
        for block in cleaned.split("\n\n") {
            let lines: Vec<&str> = block
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect();
            let Some(arrow_idx) = lines.iter().position(|l| l.contains("-->")) else {
                continue;
            };
            let timing = lines[arrow_idx];
            let Some((start_str, _)) = timing.split_once("-->") else {
                continue;
            };
            let Some(start_ms) = parse_timestamp_ms(start_str.trim()) else {
                continue;
            };
            let text = strip_tags_and_ass(&lines[arrow_idx + 1..].join(" "));
            if !text.is_empty() {
                cues.push((start_ms, text));
            }
        }
    }

    cues.sort_by_key(|(ms, _)| *ms);
    cues.into_iter()
        .enumerate()
        .map(|(index, (time_ms, text))| SubtitleCueItem {
            index,
            time_ms,
            time_label: format_cue_clock(time_ms),
            text,
        })
        .collect()
}

fn parse_timestamp_ms(input: &str) -> Option<i64> {
    let token = input.split_whitespace().next()?.replace(',', ".");
    let parts: Vec<&str> = token.split(':').collect();
    let (h, m, s) = match parts.len() {
        3 => (
            parts[0].parse::<i64>().ok()?,
            parts[1].parse::<i64>().ok()?,
            parts[2].parse::<f64>().ok()?,
        ),
        2 => (
            0,
            parts[0].parse::<i64>().ok()?,
            parts[1].parse::<f64>().ok()?,
        ),
        _ => return None,
    };
    Some(((h * 3600 + m * 60) * 1000) + (s * 1000.0).round() as i64)
}

fn strip_tags_and_ass(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_angle = false;
    let mut in_brace = false;
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '<' {
            in_angle = true;
            i += 1;
            continue;
        }
        if ch == '>' && in_angle {
            in_angle = false;
            i += 1;
            continue;
        }
        if ch == '{' {
            in_brace = true;
            i += 1;
            continue;
        }
        if ch == '}' && in_brace {
            in_brace = false;
            i += 1;
            continue;
        }
        if !in_angle && !in_brace {
            if ch == '\\' && i + 1 < chars.len() && (chars[i + 1] == 'N' || chars[i + 1] == 'n') {
                out.push(' ');
                i += 2;
                continue;
            }
            out.push(ch);
        }
        i += 1;
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn format_cue_clock(ms: i64) -> String {
    let total_secs = (ms.max(0) / 1000) as u64;
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    if h > 0 {
        format!("{h:02}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}
