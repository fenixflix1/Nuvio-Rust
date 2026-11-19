use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const DEFAULT_INTRODB_URL: &str = "https://api.introdb.app";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkipChapter {
    pub start_ms: i64,
    pub end_ms: Option<i64>,
    pub title: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkipInterval {
    pub start_time: f64,
    pub end_time: f64,
    pub skip_type: String,
    pub provider: String,
    pub skips_to_post_credits: bool,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub key: String,
}

impl SkipInterval {
    pub fn with_meta(mut self) -> Self {
        self.key = format!(
            "{}:{:.2}:{:.2}",
            self.skip_type, self.start_time, self.end_time
        );
        self.label = self.display_label().to_string();
        self
    }

    pub fn category(&self) -> &'static str {
        match self.skip_type.as_str() {
            "op" | "intro" | "mixed-op" => "opening",
            "ed" | "outro" | "credits" | "movie-credits" | "mixed-ed" => "ending",
            "recap" => "recap",
            "post-credits" => "post-credits",
            _ => "other",
        }
    }

    pub fn display_label(&self) -> &'static str {
        if self.skips_to_post_credits {
            return "Pular para pós-créditos";
        }
        match self.skip_type.as_str() {
            "op" | "intro" | "mixed-op" => "Pular abertura",
            "recap" => "Pular recapitulação",
            "ed" | "outro" | "credits" | "mixed-ed" => "Pular encerramento",
            "movie-credits" => "Pular créditos",
            "post-credits" => "Pular pós-créditos",
            _ => "Pular",
        }
    }
}

/// Converts container Matroska chapters matching `ChapterSkipIntervals.kt` rules (5s..240s).
pub fn chapters_to_skip_intervals(chapters: &[SkipChapter]) -> Vec<SkipInterval> {
    let mut intervals = Vec::new();
    let mut generic_intro = Vec::new();
    let mut has_opening = false;

    for (idx, chap) in chapters.iter().enumerate() {
        let start_sec = chap.start_ms as f64 / 1000.0;
        let next_start = chapters
            .get(idx + 1)
            .map(|c| c.start_ms as f64 / 1000.0)
            .or_else(|| chap.end_ms.map(|ms| ms as f64 / 1000.0));
        let Some(end_sec) = next_start else {
            continue;
        };
        let dur = end_sec - start_sec;
        if !(5.0..=240.0).contains(&dur) {
            continue;
        }
        let lower = chap.title.trim().to_ascii_lowercase();
        let skip_type = if matches_word_or_suffix(&lower, &["op", "opening"]) {
            has_opening = true;
            "op"
        } else if matches_word_or_suffix(&lower, &["ed", "ending"]) || matches_credits(&lower) {
            "ed"
        } else if lower == "recap" || lower.starts_with("previously") {
            "recap"
        } else if lower == "post-credits" || lower == "after-credits" || lower == "mid-credits" {
            "post-credits"
        } else if lower == "intro" || lower == "introdução" || lower == "abertura" {
            generic_intro.push(SkipInterval {
                start_time: start_sec,
                end_time: end_sec,
                skip_type: "intro".to_string(),
                provider: "chapters".to_string(),
                ..Default::default()
            });
            continue;
        } else {
            continue;
        };

        intervals.push(SkipInterval {
            start_time: start_sec,
            end_time: end_sec,
            skip_type: skip_type.to_string(),
            provider: "chapters".to_string(),
            ..Default::default()
        });
    }

    if !has_opening {
        intervals.extend(generic_intro);
    }
    intervals
}

fn matches_word_or_suffix(s: &str, bases: &[&str]) -> bool {
    let clean = s
        .replace("credits", "")
        .replace("theme", "")
        .trim()
        .to_string();
    let trimmed = clean.trim_end_matches(|c: char| c.is_ascii_digit() || c.is_whitespace());
    bases.contains(&trimmed)
}

fn matches_credits(s: &str) -> bool {
    matches!(s, "credits" | "end credits" | "closing credits" | "créditos")
}

/// Fetches skip intervals from AniSkip (resolving IMDb/TMDB/Kitsu/AniList -> MAL via ARM)
/// and IntroDB (defaulting to https://api.introdb.app), merging with container chapters.
#[allow(clippy::too_many_arguments)]
pub async fn resolve_skip_intervals(
    video_id: &str,
    tmdb_id: Option<&str>,
    media_type: &str,
    season: Option<i32>,
    episode: Option<i32>,
    introdb_url: &str,
    tmdb_api_key: &str,
    aniskip_enabled: bool,
    chapter_intervals: Vec<SkipInterval>,
) -> Vec<SkipInterval> {
    let client = match Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return apply_post_credits_rules(chapter_intervals),
    };

    let (mut imdb_id, resolved_season, resolved_episode, mut mal_id, tmdb_num, kitsu_id, anilist_id) =
        parse_lookup_ids(video_id, tmdb_id, season, episode);

    // Resolve missing IMDb / MAL IDs from TMDB or ARM when available so anime/series work with
    // both IntroDB and AniSkip regardless of catalog ID prefix (tt / tmdb / kitsu / mal).
    if let Some(ref k) = kitsu_id {
        if let Some((arm_imdb, arm_mal)) = fetch_arm_ids(&client, "kitsu", k).await {
            if imdb_id.is_none() {
                imdb_id = arm_imdb;
            }
            if mal_id.is_none() {
                mal_id = arm_mal;
            }
        }
    } else if let Some(ref al) = anilist_id {
        if let Some((arm_imdb, arm_mal)) = fetch_arm_ids(&client, "anilist", al).await {
            if imdb_id.is_none() {
                imdb_id = arm_imdb;
            }
            if mal_id.is_none() {
                mal_id = arm_mal;
            }
        }
    } else if let Some(ref tid) = tmdb_num {
        if let Some((arm_imdb, arm_mal)) = fetch_arm_ids(&client, "themoviedb", tid).await {
            if imdb_id.is_none() {
                imdb_id = arm_imdb;
            }
            if mal_id.is_none() {
                mal_id = arm_mal;
            }
        }
        if imdb_id.is_none() && !tmdb_api_key.trim().is_empty() {
            imdb_id = fetch_tmdb_external_imdb(&client, tid, media_type, tmdb_api_key).await;
        }
    }

    if aniskip_enabled && mal_id.is_none() && let Some(ref imdb) = imdb_id
        && let Some((_, arm_mal)) = fetch_arm_ids(&client, "imdb", imdb).await
    {
        mal_id = arm_mal;
    }

    let trimmed_intro = introdb_url.trim().trim_end_matches('/');
    let intro_base = if trimmed_intro.is_empty() {
        DEFAULT_INTRODB_URL
    } else {
        trimmed_intro
    };
    let intro_fut = async {
        let Some(ref imdb) = imdb_id else {
            return Vec::new();
        };
        let url = if media_type == "movie" {
            format!("{intro_base}/segments?imdb_id={imdb}&is_movie=true")
        } else if let (Some(s), Some(e)) = (resolved_season, resolved_episode) {
            format!("{intro_base}/segments?imdb_id={imdb}&season={s}&episode={e}")
        } else {
            return Vec::new();
        };
        fetch_introdb_segments(&client, &url, media_type == "movie").await
    };

    let aniskip_fut = async {
        if !aniskip_enabled {
            return Vec::new();
        }
        let (Some(mal), Some(ep)) = (mal_id, resolved_episode.or(Some(1))) else {
            return Vec::new();
        };
        fetch_aniskip_segments(&client, &mal, ep).await
    };

    let (introdb_list, aniskip_list) = tokio::join!(intro_fut, aniskip_fut);

    let mut api_intervals = introdb_list;
    let mut covered_categories: std::collections::HashSet<&'static str> =
        api_intervals.iter().map(|i| i.category()).collect();
    for item in aniskip_list {
        if !covered_categories.contains(item.category()) {
            covered_categories.insert(item.category());
            api_intervals.push(item);
        }
    }

    let merged = if media_type == "movie" {
        if !api_intervals.is_empty() {
            api_intervals
        } else {
            chapter_intervals
        }
    } else if chapter_intervals.is_empty() {
        api_intervals
    } else {
        let mut out = chapter_intervals;
        let chap_cats: std::collections::HashSet<&'static str> =
            out.iter().map(|i| i.category()).collect();
        for item in api_intervals {
            if !chap_cats.contains(item.category()) {
                out.push(item);
            }
        }
        out
    };

    apply_post_credits_rules(merged)
}

fn apply_post_credits_rules(mut intervals: Vec<SkipInterval>) -> Vec<SkipInterval> {
    let post_start = intervals
        .iter()
        .filter(|i| i.skip_type == "post-credits")
        .map(|i| i.start_time)
        .reduce(f64::min);

    if let Some(ps) = post_start {
        for item in &mut intervals {
            if matches!(
                item.skip_type.as_str(),
                "ed" | "outro" | "credits" | "movie-credits" | "mixed-ed"
            ) && item.start_time < ps
                && item.end_time > ps
            {
                item.end_time = ps;
                item.skips_to_post_credits = true;
            }
        }
    }
    intervals.retain(|i| i.end_time > i.start_time + 1.0);
    intervals.sort_by(|a, b| {
        a.start_time
            .partial_cmp(&b.start_time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    intervals.into_iter().map(SkipInterval::with_meta).collect()
}

#[allow(clippy::type_complexity)]
fn parse_lookup_ids(
    video_id: &str,
    tmdb_hint: Option<&str>,
    season: Option<i32>,
    episode: Option<i32>,
) -> (
    Option<String>,
    Option<i32>,
    Option<i32>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let clean_tmdb_hint = tmdb_hint
        .map(|s| s.trim().trim_start_matches("tmdb:"))
        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
        .map(str::to_string);
    let parts: Vec<&str> = video_id.split(':').collect();
    if let Some(mal) = video_id.strip_prefix("mal:") {
        let mut mparts = mal.split(':');
        let id = mparts.next().map(str::to_string);
        let ep = mparts.next().and_then(|s| s.parse::<i32>().ok()).or(episode);
        return (None, season, ep, id, clean_tmdb_hint, None, None);
    }
    if let Some(kitsu) = video_id.strip_prefix("kitsu:") {
        let mut kparts = kitsu.split(':');
        let id = kparts.next().map(str::to_string);
        let ep = kparts.next().and_then(|s| s.parse::<i32>().ok()).or(episode);
        return (None, season, ep, None, clean_tmdb_hint, id, None);
    }
    if let Some(al) = video_id.strip_prefix("anilist:") {
        let mut aparts = al.split(':');
        let id = aparts.next().map(str::to_string);
        let ep = aparts.next().and_then(|s| s.parse::<i32>().ok()).or(episode);
        return (None, season, ep, None, clean_tmdb_hint, None, id);
    }
    if let Some(tmdb) = video_id.strip_prefix("tmdb:") {
        let mut tparts = tmdb.split(':');
        let id = tparts.next().map(str::to_string).or(clean_tmdb_hint);
        let s = tparts.next().and_then(|v| v.parse::<i32>().ok()).or(season);
        let e = tparts.next().and_then(|v| v.parse::<i32>().ok()).or(episode);
        return (None, s, e, None, id, None, None);
    }
    if parts.first().is_some_and(|p| p.starts_with("tt")) {
        let imdb = Some(parts[0].to_string());
        let s = parts.get(1).and_then(|v| v.parse::<i32>().ok()).or(season);
        let e = parts.get(2).and_then(|v| v.parse::<i32>().ok()).or(episode);
        return (imdb, s, e, None, clean_tmdb_hint, None, None);
    }
    if parts
        .first()
        .is_some_and(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
    {
        let tid = Some(parts[0].to_string());
        let s = parts.get(1).and_then(|v| v.parse::<i32>().ok()).or(season);
        let e = parts.get(2).and_then(|v| v.parse::<i32>().ok()).or(episode);
        return (None, s, e, None, tid, None, None);
    }
    (None, season, episode, None, clean_tmdb_hint, None, None)
}

async fn fetch_arm_ids(
    client: &Client,
    source: &str,
    id: &str,
) -> Option<(Option<String>, Option<String>)> {
    let url = format!("https://arm.haglund.dev/api/v2/{source}?id={id}");
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let val: Value = resp.json().await.ok()?;
    let item = val
        .as_array()
        .and_then(|arr| arr.first())
        .or_else(|| val.is_object().then_some(&val))?;
    let imdb = item
        .get("imdb")
        .and_then(|v| v.as_str())
        .filter(|s| s.starts_with("tt"))
        .map(str::to_string);
    let mal = item
        .get("myanimelist")
        .and_then(|v| v.as_i64().or_else(|| v.as_u64().map(|u| u as i64)))
        .filter(|&n| n > 0)
        .map(|n| n.to_string());
    Some((imdb, mal))
}

async fn fetch_tmdb_external_imdb(
    client: &Client,
    tmdb_id: &str,
    media_type: &str,
    api_key: &str,
) -> Option<String> {
    let t = if media_type == "movie" { "movie" } else { "tv" };
    let url = format!(
        "https://api.themoviedb.org/3/{t}/{tmdb_id}/external_ids?api_key={}",
        api_key.trim()
    );
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let val: Value = resp.json().await.ok()?;
    val.get("imdb_id")
        .and_then(|v| v.as_str())
        .filter(|s| s.starts_with("tt"))
        .map(str::to_string)
}

async fn fetch_introdb_segments(client: &Client, url: &str, is_movie: bool) -> Vec<SkipInterval> {
    let Ok(resp) = client.get(url).send().await else {
        return Vec::new();
    };
    if !resp.status().is_success() {
        return Vec::new();
    }
    let Ok(val) = resp.json::<Value>().await else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let pairs = [
        ("intro", "intro"),
        ("recap", "recap"),
        ("outro", if is_movie { "movie-credits" } else { "outro" }),
        ("post_credits", "post-credits"),
    ];
    for (field, skip_type) in pairs {
        if let Some(seg) = val.get(field).and_then(|v| v.as_object())
            && let (Some(s), Some(e)) = (parse_seg_time(seg, "start"), parse_seg_time(seg, "end"))
            && e > s
        {
            out.push(SkipInterval {
                start_time: s,
                end_time: e,
                skip_type: skip_type.to_string(),
                provider: "introdb".to_string(),
                ..Default::default()
            });
        }
    }
    out
}

fn parse_seg_time(obj: &serde_json::Map<String, Value>, prefix: &str) -> Option<f64> {
    if let Some(sec) = obj.get(&format!("{prefix}_sec")).and_then(|v| v.as_f64()) {
        return Some(sec);
    }
    obj.get(&format!("{prefix}_ms"))
        .and_then(|v| v.as_f64())
        .map(|ms| ms / 1000.0)
}

async fn fetch_aniskip_segments(client: &Client, mal_id: &str, ep: i32) -> Vec<SkipInterval> {
    let url = format!(
        "https://api.aniskip.com/v2/skip-times/{mal_id}/{ep}?types=op&types=ed&types=recap&types=mixed-op&types=mixed-ed&episodeLength=0"
    );
    let Ok(resp) = client.get(url).send().await else {
        return Vec::new();
    };
    if !resp.status().is_success() {
        return Vec::new();
    }
    let Ok(val) = resp.json::<Value>().await else {
        return Vec::new();
    };
    let Some(results) = val.get("results").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for res in results {
        let Some(skip_type) = res.get("skipType").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(interval) = res.get("interval").and_then(|v| v.as_object()) else {
            continue;
        };
        let Some(start) = interval.get("startTime").and_then(|v| v.as_f64()) else {
            continue;
        };
        let Some(end) = interval.get("endTime").and_then(|v| v.as_f64()) else {
            continue;
        };
        if end > start {
            out.push(SkipInterval {
                start_time: start,
                end_time: end,
                skip_type: skip_type.to_string(),
                provider: "aniskip".to_string(),
                ..Default::default()
            });
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
pub async fn submit_introdb_segment(
    introdb_url: &str,
    api_key: &str,
    video_id: &str,
    season: Option<i32>,
    episode: Option<i32>,
    start_sec: f64,
    end_sec: f64,
    segment_type: &str,
) -> Result<(), String> {
    let trimmed = introdb_url.trim().trim_end_matches('/');
    let base = if trimmed.is_empty() {
        DEFAULT_INTRODB_URL
    } else {
        trimmed
    };
    if api_key.trim().is_empty() {
        return Err("Configure sua API Key do IntroDB em Ajustes → Player → Pular Abertura.".to_string());
    }
    let (imdb_opt, rs, re, _, _, _, _) = parse_lookup_ids(video_id, None, season, episode);
    let imdb_id = imdb_opt.ok_or_else(|| "IMDb ID indisponível para envio.".to_string())?;
    if end_sec <= start_sec {
        return Err("Tempo final deve ser maior que o tempo inicial.".to_string());
    }
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())?;
    let payload = serde_json::json!({
        "imdb_id": imdb_id,
        "season": rs.unwrap_or(1),
        "episode": re.unwrap_or(1),
        "start_sec": start_sec,
        "end_sec": end_sec,
        "start_ms": (start_sec * 1000.0).round() as i64,
        "end_ms": (end_sec * 1000.0).round() as i64,
        "segment_type": segment_type,
    });
    let resp = client
        .post(format!("{base}/submit"))
        .header("Authorization", format!("Bearer {}", api_key.trim()))
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.status().is_success() {
        Ok(())
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Err(format!("IntroDB HTTP {status}: {body}"))
    }
}
