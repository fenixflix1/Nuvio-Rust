use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlaybackOptions {
    pub subtitle_sync: bool,
    pub thorough_sync: bool,
    pub upscale: String,
    pub upscale_mode: String,
    pub svp: bool,
    pub custom_enabled: bool,
    pub custom_config: String,
}

impl Default for PlaybackOptions {
    fn default() -> Self {
        Self {
            subtitle_sync: false,
            thorough_sync: true,
            upscale: "off".into(),
            upscale_mode: "soft".into(),
            svp: false,
            custom_enabled: false,
            custom_config: String::new(),
        }
    }
}

pub struct VideoOptions {
    pub options: Vec<(&'static str, &'static str)>,
    pub config: Option<PathBuf>,
    pub alang: Option<String>,
    pub slang: Option<String>,
}

fn normalize_lang_token(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut prev_dash = false;
    for ch in raw.trim().chars() {
        let lower = match ch {
            'á' | 'à' | 'â' | 'ã' | 'Á' | 'À' | 'Â' | 'Ã' => 'a',
            'é' | 'ê' | 'è' | 'É' | 'Ê' | 'È' => 'e',
            'í' | 'ì' | 'î' | 'Í' | 'Ì' | 'Î' => 'i',
            'ó' | 'ô' | 'õ' | 'ò' | 'Ó' | 'Ô' | 'Õ' | 'Ò' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
            'ç' | 'Ç' => 'c',
            'ñ' | 'Ñ' => 'n',
            _ => ch.to_ascii_lowercase(),
        };
        if lower == '_' || lower.is_whitespace() || lower == '-' {
            if !out.is_empty() && !prev_dash {
                out.push('-');
                prev_dash = true;
            }
        } else if lower.is_ascii_alphanumeric() {
            out.push(lower);
            prev_dash = false;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn expand_single_lang_token(token: &str) -> Vec<String> {
    let expanded: &[&str] = match token {
        "pt-br" | "ptbr" | "ptb" | "pob" | "portugues-br" | "portugues-brasil"
        | "portuguese-br" | "portuguese-brazil" | "brasil" | "brazil" | "brazilian"
        | "dublado" => &["pt-br", "ptb", "pob", "por", "pt"],
        "pt-pt" | "portugues-pt" | "portuguese-pt" | "portugal" => &["pt-pt", "por", "pt"],
        "pt" | "por" | "portugues" | "portuguese" => {
            &["pt-br", "ptb", "pob", "por", "pt", "pt-pt"]
        }
        "en-us" => &["en-us", "en", "eng"],
        "en-gb" => &["en-gb", "en", "eng"],
        "en" | "eng" | "english" | "ingles" => &["en", "eng"],
        "ja" | "jp" | "jpn" | "japanese" | "japones" => &["jpn", "ja", "jp"],
        "es-la" | "es-419" | "latino" => &["es-la", "es-419", "spa", "es"],
        "es-es" | "castellano" => &["es-es", "spa", "es"],
        "es" | "spa" | "spanish" | "espanhol" | "espanol" => {
            &["es-la", "es-419", "spa", "es", "es-es"]
        }
        "fr" | "fre" | "fra" | "french" | "frances" => &["fra", "fre", "fr"],
        "de" | "ger" | "deu" | "german" | "alemao" => &["deu", "ger", "de"],
        "it" | "ita" | "italian" | "italiano" => &["ita", "it"],
        "ko" | "kor" | "korean" | "coreano" => &["kor", "ko"],
        "zh-cn" => &["zh-cn", "zho", "chi", "zh"],
        "zh-tw" => &["zh-tw", "zho", "chi", "zh"],
        "zh" | "zho" | "chi" | "chinese" | "chines" => &["zho", "chi", "zh"],
        "ru" | "rus" | "russian" | "russo" => &["rus", "ru"],
        _ => {
            let mut list = vec![token.to_string()];
            if let Some((base, _)) = token.split_once('-')
                && !base.is_empty()
            {
                list.push(base.to_string());
            }
            return list;
        }
    };
    expanded.iter().map(|s| (*s).to_string()).collect()
}

/// Expands a user language preference (e.g. `"pt-br"`, `"por"`, `"pt-br, eng"`)
/// into a deduplicated comma-separated mpv `alang`/`slang` priority list.
pub fn expand_mpv_lang_list(pref: &str) -> Option<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for raw_part in pref.split([',', ';']) {
        let token = normalize_lang_token(raw_part);
        if token.is_empty() {
            continue;
        }
        for code in expand_single_lang_token(&token) {
            if seen.insert(code.clone()) {
                out.push(code);
            }
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out.join(","))
    }
}

impl VideoOptions {
    pub fn with_languages(mut self, audio_lang: &str, sub_lang: &str) -> Self {
        self.alang = expand_mpv_lang_list(audio_lang);
        self.slang = expand_mpv_lang_list(sub_lang);
        self
    }
}

impl PlaybackOptions {
    pub fn prepare(&self, anime: bool, cache: &Path) -> Result<VideoOptions, String> {
        let mut options = Vec::new();
        if self.upscale == "always" || (self.upscale == "anime" && anime) {
            let scaler = match self.upscale_mode.as_str() {
                "light" => "spline36",
                "sharp" => "ewa_lanczossharp",
                _ => "ewa_lanczos",
            };
            options.extend([
                ("scale", scaler),
                ("cscale", scaler),
                ("sigmoid-upscaling", "yes"),
            ]);
        }
        if self.svp {
            let socket = if cfg!(windows) {
                "mpvpipe"
            } else {
                "/tmp/mpvsocket"
            };
            options.extend([
                ("input-ipc-server", socket),
                ("hwdec", "auto-copy"),
                ("hwdec-codecs", "all"),
                ("hr-seek-framedrop", "no"),
            ]);
        }
        let config = if self.custom_enabled && !self.custom_config.trim().is_empty() {
            std::fs::create_dir_all(cache).map_err(|e| e.to_string())?;
            let path = cache.join("custom-mpv.conf");
            std::fs::write(&path, &self.custom_config).map_err(|e| e.to_string())?;
            Some(path)
        } else {
            None
        };
        Ok(VideoOptions {
            options,
            config,
            alang: None,
            slang: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::expand_mpv_lang_list;

    #[test]
    fn expands_pt_br_and_por_equivalently_for_mpv() {
        assert_eq!(
            expand_mpv_lang_list("pt-br").as_deref(),
            Some("pt-br,ptb,pob,por,pt")
        );
        assert_eq!(
            expand_mpv_lang_list("pt br").as_deref(),
            Some("pt-br,ptb,pob,por,pt")
        );
        assert_eq!(
            expand_mpv_lang_list("pt_BR").as_deref(),
            Some("pt-br,ptb,pob,por,pt")
        );
        assert_eq!(
            expand_mpv_lang_list("por").as_deref(),
            Some("pt-br,ptb,pob,por,pt,pt-pt")
        );
        assert_eq!(
            expand_mpv_lang_list("Português").as_deref(),
            Some("pt-br,ptb,pob,por,pt,pt-pt")
        );
    }

    #[test]
    fn expands_multiple_languages_and_ignores_empty() {
        assert_eq!(expand_mpv_lang_list("   "), None);
        assert_eq!(
            expand_mpv_lang_list("pt-br, eng").as_deref(),
            Some("pt-br,ptb,pob,por,pt,en,eng")
        );
        assert_eq!(
            expand_mpv_lang_list("ja").as_deref(),
            Some("jpn,ja,jp")
        );
    }
}
