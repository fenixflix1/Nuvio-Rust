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
        Ok(VideoOptions { options, config })
    }
}
