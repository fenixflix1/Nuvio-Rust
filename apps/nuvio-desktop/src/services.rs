use nuvio_core::integrations::{mdblist::MdbListClient, shared_http_client, tmdb::TmdbClient};
use nuvio_core::plugins::resolve_imdb_to_tmdb;
use nuvio_core::storage::CacheStore;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Services {
    pub discord_enabled: bool,
    pub discord_style: String,
    pub discord_scope: String,
    pub portrait: bool,
    pub episode_image: bool,
    pub episode_name: bool,
    pub tmdb_enabled: bool,
    pub tmdb_language: String,
    pub tmdb_text: bool,
    pub tmdb_images: bool,
    pub tmdb_details: bool,
    pub tmdb_credits: bool,
    pub tmdb_trailers: bool,
    pub tmdb_productions: bool,
    pub tmdb_networks: bool,
    pub tmdb_episodes: bool,
    pub tmdb_season_posters: bool,
    pub tmdb_similar: bool,
    pub tmdb_collections: bool,
    pub mdblist_hero: bool,
    pub mdblist_enabled: bool,
    pub mdblist_key: String,
    pub providers: Vec<String>,
}

impl Default for Services {
    fn default() -> Self {
        Self {
            discord_enabled: false,
            discord_style: "watching".into(),
            discord_scope: "all".into(),
            portrait: false,
            episode_image: true,
            episode_name: true,
            tmdb_enabled: false,
            tmdb_language: "pt-BR".into(),
            tmdb_text: true,
            tmdb_images: true,
            tmdb_details: true,
            tmdb_credits: true,
            tmdb_trailers: true,
            tmdb_productions: true,
            tmdb_networks: true,
            tmdb_episodes: true,
            tmdb_season_posters: true,
            tmdb_similar: true,
            tmdb_collections: true,
            mdblist_hero: false,
            mdblist_enabled: false,
            mdblist_key: String::new(),
            providers: [
                "imdb",
                "tmdb",
                "tomatoes",
                "metacritic",
                "trakt",
                "letterboxd",
                "audience",
                "mal",
            ]
            .map(String::from)
            .to_vec(),
        }
    }
}

impl Services {
    pub async fn enrich(&self, meta: &mut Value, kind: &str, id: &str, key: &str) {
        if !matches!(kind, "movie" | "series" | "tv") {
            return;
        }
        let mut tid = ["moviedb_id", "tmdb_id"]
            .iter()
            .find_map(|field| {
                let val = &meta[*field];
                val.as_u64().or_else(|| val.as_str()?.parse().ok())
            })
            .or_else(|| id.strip_prefix("tmdb:")?.parse().ok());
        let imdb = [Some(id), meta["imdb_id"].as_str(), meta["imdbId"].as_str()]
            .into_iter()
            .flatten()
            .find(|s| {
                s.strip_prefix("tt")
                    .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
            })
            .map(String::from);
        if !key.is_empty() && (self.tmdb_enabled || (self.tmdb_collections && kind == "movie")) {
            if let Ok(client) = TmdbClient::new(key, &self.tmdb_language) {
                if tid.is_none()
                    && let Some(imdb) = &imdb
                {
                    tid = resolve_imdb_to_tmdb(imdb, kind, key)
                        .await
                        .and_then(|n| n.parse().ok());
                }
                if tid.is_none()
                    && kind == "movie"
                    && let Some(title) = meta["name"].as_str().map(str::trim).filter(|s| !s.is_empty())
                    && let Ok(search) = client.search_multi(title, 1).await
                {
                    tid = search
                        .results
                        .into_iter()
                        .find(|r| r.media_type.as_deref().unwrap_or("movie") == "movie")
                        .map(|r| r.id);
                }
                if let Some(tid) = tid
                    && let Ok(data) = client.details(kind, tid).await
                {
                    meta["tmdb_id"] = json!(tid);
                    if self.tmdb_enabled {
                        self.merge_tmdb(meta, &data, kind);
                    }
                    if self.tmdb_collections
                        && let Some(id) = data["belongs_to_collection"]["id"].as_u64()
                        && let Ok(collection) = client.collection(id).await
                    {
                        meta["tmdbCollection"] = collection_cards(&collection["parts"]);
                        meta["collectionName"] = collection["name"].clone();
                    }
                }
            }
        }
        if self.mdblist_enabled && !self.mdblist_key.is_empty() && !self.providers.is_empty() {
            let lookup = imdb
                .map(|id| ("imdb", id))
                .or_else(|| tid.map(|n| ("tmdb", n.to_string())));
            if let Some((provider, id)) = lookup
                && let Ok(client) = MdbListClient::new(&self.mdblist_key)
                && let Ok(rows) = client.ratings(provider, kind, &id).await
            {
                meta["externalRatings"] = self.ratings(&rows);
            }
        }
    }

    fn ratings(&self, rows: &[Value]) -> Value {
        let mut result = Vec::new();
        for row in rows {
            let source = match row["source"].as_str().unwrap_or("") {
                "popcorn" | "tomatoesaudience" => "audience",
                "myanimelist" => "mal",
                src => src,
            };
            let maximum = match source {
                "imdb" | "mal" => 10.0,
                "letterboxd" => 5.0,
                _ => 100.0,
            };
            let raw = row["value"].as_f64().filter(|n| *n >= 0.);
            let score = row["score"]
                .as_f64()
                .filter(|n| (0.0..=100.).contains(n))
                .map(|n| n * maximum / 100.);
            let value = if source == "letterboxd" {
                score.or(raw)
            } else {
                raw.or(score)
            };
            if let Some(value) = value.filter(|n| (0.0..=maximum).contains(n))
                && self.providers.iter().any(|s| s == source)
                && !result.iter().any(|r: &Value| r["source"] == source)
            {
                result.push(json!({"source":source,"value":value,"maximum":maximum}));
            }
        }
        json!(result)
    }

    fn merge_tmdb(&self, meta: &mut Value, data: &Value, kind: &str) {
        copy_text(meta, "imdb_id", &data["external_ids"]["imdb_id"]);
        if self.tmdb_text {
            for (src, dst) in [
                ("title", "name"),
                ("name", "name"),
                ("overview", "description"),
                ("release_date", "releaseInfo"),
                ("first_air_date", "releaseInfo"),
            ] {
                copy_text(meta, dst, &data[src]);
            }
        }
        if self.tmdb_images {
            for (src, dst, size) in [
                ("poster_path", "poster", "w500"),
                ("backdrop_path", "background", "w1280"),
            ] {
                if let Some(url) = image(&data[src], size) {
                    meta[dst] = json!(url);
                }
            }
            if let Some(logo) = data["images"]["logos"]
                .get(0)
                .and_then(|l| image(&l["file_path"], "w500"))
            {
                meta["logo"] = json!(logo);
            }
        }
        if self.tmdb_details {
            copy_text(meta, "status", &data["status"]);
            copy_text(meta, "language", &data["original_language"]);
            let countries = country_codes(&data["production_countries"], &data["origin_country"]);
            if !countries.is_empty() {
                meta["country"] = json!(countries.join(", "));
            }
            if let Some(cert) = select_age_rating(data, kind, &self.tmdb_language) {
                meta["ageRating"] = json!(cert);
            }
            let genres = names(&data["genres"]);
            if !genres.is_empty() {
                meta["genres"] = json!(genres);
            }
            if let Some(n) = data["runtime"]
                .as_u64()
                .or_else(|| data["episode_run_time"][0].as_u64())
                .filter(|n| *n > 0)
            {
                meta["runtime"] = json!(format!("{n} min"));
            }
        }
        if self.tmdb_credits {
            let people = build_people(data, kind);
            if !people.is_empty() {
                meta["castMembers"] = json!(people);
            }
            let dirs = build_director_names(data, kind);
            if !dirs.is_empty() {
                meta["director"] = json!(dirs);
            }
        }
        for (enabled, src, str_dst, list_dst) in [
            (
                self.tmdb_productions,
                "production_companies",
                "production",
                "productionCompanies",
            ),
            (
                self.tmdb_networks,
                "networks",
                "networks",
                "networkCompanies",
            ),
        ] {
            if !enabled {
                continue;
            }
            let names = names(&data[src]);
            if !names.is_empty() {
                meta[str_dst] = json!(names.join(", "));
            }
            let companies = build_companies(&data[src]);
            if !companies.is_empty() {
                meta[list_dst] = json!(companies);
            }
        }
        if self.tmdb_trailers
            && let Some(videos) = data["videos"]["results"].as_array()
        {
            let mut trailers: Vec<_> = videos
                .iter()
                .filter(|v| v["site"] == "YouTube")
                .filter_map(|v| {
                    let key = v["key"].as_str()?.trim();
                    if key.is_empty() {
                        return None;
                    }
                    Some(json!({
                        "source": key,
                        "key": key,
                        "name": v["name"].as_str().unwrap_or(""),
                        "type": v["type"].as_str().unwrap_or("Trailer"),
                        "publishedAt": v["published_at"].as_str().unwrap_or(""),
                    }))
                })
                .collect();
            trailers.sort_by_key(|v| if v["type"] == "Trailer" { 0 } else { 1 });
            if !trailers.is_empty() {
                meta["trailers"] = json!(trailers);
            }
        }
        if self.tmdb_similar {
            meta["tmdbSimilar"] = cards(
                &data["recommendations"]["results"],
                kind,
                data["id"].as_u64().unwrap_or_default(),
            );
        }
        if (self.tmdb_season_posters || self.tmdb_episodes)
            && let Some(seasons) = data["seasons"].as_array()
        {
            meta["tmdbSeasons"] = json!(seasons.iter().filter_map(|s| Some(json!({
                "number":s["season_number"].as_u64()?,
                "poster": if self.tmdb_season_posters { image(&s["poster_path"],"w185") } else { None }
            }))).collect::<Vec<_>>());
        }
    }
}

fn copy_text(meta: &mut Value, dst: &str, value: &Value) {
    if value.as_str().is_some_and(|s| !s.trim().is_empty()) {
        meta[dst] = value.clone();
    }
}

fn names(items: &Value) -> Vec<&str> {
    items
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v["name"].as_str())
        .collect()
}

fn country_codes(production_countries: &Value, origin_country: &Value) -> Vec<String> {
    let iso: Vec<String> = production_countries
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| c["iso_3166_1"].as_str().map(str::trim))
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if !iso.is_empty() {
        return iso;
    }
    let origins: Vec<String> = origin_country
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| c.as_str().map(str::trim))
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if !origins.is_empty() {
        return origins;
    }
    names(production_countries)
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn preferred_age_regions(language: &str) -> Vec<String> {
    let mut regions = Vec::new();
    let mut parts = language.trim().split('-');
    let lang = parts.next().unwrap_or("").to_ascii_lowercase();
    if let Some(reg) = parts.next() {
        let up = reg.trim().to_ascii_uppercase();
        if up.len() == 2 {
            regions.push(up);
        }
    } else if !lang.is_empty() {
        let mapped = match lang.as_str() {
            "pt" => Some("BR"),
            "es" => Some("ES"),
            "fr" => Some("FR"),
            "de" => Some("DE"),
            "it" => Some("IT"),
            "ja" => Some("JP"),
            "ko" => Some("KR"),
            _ => None,
        };
        if let Some(m) = mapped {
            regions.push(m.to_string());
        }
    }
    for fallback in ["US", "GB"] {
        if !regions.iter().any(|r| r == fallback) {
            regions.push(fallback.to_string());
        }
    }
    regions
}

fn select_age_rating(data: &Value, kind: &str, language: &str) -> Option<String> {
    let regions = preferred_age_regions(language);
    if matches!(kind, "series" | "tv") {
        let results = data["content_ratings"]["results"].as_array()?;
        for region in &regions {
            if let Some(found) = results.iter().find(|r| {
                r["iso_3166_1"]
                    .as_str()
                    .is_some_and(|iso| iso.eq_ignore_ascii_case(region))
            }) && let Some(cert) = found["rating"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                return Some(cert.to_string());
            }
        }
        return results
            .iter()
            .filter_map(|r| r["rating"].as_str().map(str::trim))
            .find(|s| !s.is_empty())
            .map(str::to_string);
    }

    let results = data["release_dates"]["results"].as_array()?;
    for region in &regions {
        if let Some(found) = results.iter().find(|r| {
            r["iso_3166_1"]
                .as_str()
                .is_some_and(|iso| iso.eq_ignore_ascii_case(region))
        }) && let Some(cert) = found["release_dates"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|d| d["certification"].as_str().map(str::trim))
            .find(|s| !s.is_empty())
        {
            return Some(cert.to_string());
        }
    }
    results
        .iter()
        .flat_map(|r| r["release_dates"].as_array().into_iter().flatten())
        .filter_map(|d| d["certification"].as_str().map(str::trim))
        .find(|s| !s.is_empty())
        .map(str::to_string)
}

fn build_companies(items: &Value) -> Vec<Value> {
    items
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| {
            let name = c["name"].as_str()?.trim();
            if name.is_empty() {
                return None;
            }
            Some(json!({
                "id": c["id"],
                "name": name,
                "logo": image(&c["logo_path"], "w300"),
            }))
        })
        .collect()
}

fn build_director_names(data: &Value, kind: &str) -> Vec<String> {
    let mut out = Vec::new();
    if matches!(kind, "series" | "tv") {
        for c in data["created_by"].as_array().into_iter().flatten() {
            if let Some(name) = c["name"].as_str().map(str::trim).filter(|s| !s.is_empty())
                && !out.iter().any(|n: &String| n.eq_ignore_ascii_case(name))
            {
                out.push(name.to_string());
            }
        }
        return out;
    }
    for c in data["credits"]["crew"].as_array().into_iter().flatten() {
        if c["job"]
            .as_str()
            .is_some_and(|j| j.eq_ignore_ascii_case("Director"))
            && let Some(name) = c["name"].as_str().map(str::trim).filter(|s| !s.is_empty())
            && !out.iter().any(|n: &String| n.eq_ignore_ascii_case(name))
        {
            out.push(name.to_string());
        }
    }
    out
}

fn push_person(
    combined: &mut Vec<Value>,
    seen: &mut std::collections::HashSet<String>,
    id: &Value,
    name: Option<&str>,
    role: &str,
    profile_path: &Value,
) {
    let Some(name) = name.map(str::trim).filter(|s| !s.is_empty()) else {
        return;
    };
    let key = format!("{}|{}", name.to_lowercase(), role.to_lowercase());
    if !seen.insert(key) {
        return;
    }
    combined.push(json!({
        "id": id,
        "name": name,
        "character": role,
        "photo": image(profile_path, "w185"),
    }));
}

fn build_people(data: &Value, kind: &str) -> Vec<Value> {
    let is_series = matches!(kind, "series" | "tv");
    let mut combined: Vec<Value> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    if is_series {
        for c in data["created_by"].as_array().into_iter().flatten() {
            push_person(
                &mut combined,
                &mut seen,
                &c["id"],
                c["name"].as_str(),
                "Criador",
                &c["profile_path"],
            );
        }
    }
    if combined.is_empty() {
        for c in data["credits"]["crew"].as_array().into_iter().flatten() {
            if c["job"]
                .as_str()
                .is_some_and(|j| j.eq_ignore_ascii_case("Director"))
            {
                push_person(
                    &mut combined,
                    &mut seen,
                    &c["id"],
                    c["name"].as_str(),
                    "Diretor",
                    &c["profile_path"],
                );
            }
        }
    }
    if combined.is_empty() {
        for c in data["credits"]["crew"].as_array().into_iter().flatten() {
            let job = c["job"].as_str().unwrap_or("").to_ascii_lowercase();
            if job.contains("writer") || job.contains("screenplay") {
                push_person(
                    &mut combined,
                    &mut seen,
                    &c["id"],
                    c["name"].as_str(),
                    "Roteirista",
                    &c["profile_path"],
                );
            }
        }
    }

    for c in data["credits"]["cast"]
        .as_array()
        .into_iter()
        .flatten()
        .take(24)
    {
        let role = c["character"].as_str().unwrap_or("").trim();
        push_person(
            &mut combined,
            &mut seen,
            &c["id"],
            c["name"].as_str(),
            role,
            &c["profile_path"],
        );
    }

    combined
}

fn image(path: &Value, size: &str) -> Option<String> {
    let path = path.as_str()?.strip_prefix('/')?;
    Some(format!("https://image.tmdb.org/t/p/{size}/{path}"))
}

fn cards(items: &Value, kind: &str, current_id: u64) -> Value {
    json!(items.as_array().into_iter().flatten().filter_map(|v| {
        let id = v["id"].as_u64()?;
        if id == current_id { return None; }
        Some(json!({
            "id": format!("tmdb:{id}"), "type":if kind == "movie" {"movie"} else {"series"},
            "name":v["title"].as_str().or_else(||v["name"].as_str())?,
            "description":v["overview"],"poster":image(&v["poster_path"],"w342"),"banner":image(&v["backdrop_path"],"w1280")
        }))
    }).collect::<Vec<_>>())
}

fn collection_cards(items: &Value) -> Value {
    let mut parts: Vec<&Value> = items.as_array().into_iter().flatten().collect();
    parts.sort_by(|a, b| {
        let da = a["release_date"]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("9999-99-99");
        let db = b["release_date"]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("9999-99-99");
        da.cmp(db)
    });
    json!(parts.into_iter().filter_map(|v| {
        let id = v["id"].as_u64()?;
        let name = v["title"].as_str().or_else(|| v["name"].as_str())?.trim();
        if name.is_empty() {
            return None;
        }
        let release_date = v["release_date"].as_str().unwrap_or("").trim();
        let release_year = release_date.get(..4).filter(|y| y.chars().all(|c| c.is_ascii_digit()));
        let poster = image(&v["poster_path"], "w500").or_else(|| image(&v["backdrop_path"], "w780"));
        let banner = image(&v["backdrop_path"], "w1280").or_else(|| image(&v["poster_path"], "w780"));
        let rating = v["vote_average"].as_f64().filter(|r| *r > 0.0).map(|r| format!("{r:.1}"));
        Some(json!({
            "id": format!("tmdb:{id}"),
            "type": "movie",
            "name": name,
            "description": v["overview"],
            "poster": poster,
            "banner": banner,
            "release_info": release_year,
            "imdb_rating": rating,
        }))
    }).collect::<Vec<_>>())
}

/// Full actor page: profile, biography and a filmography list built from the
/// person's combined movie/TV credits.
#[tauri::command]
pub async fn person_details(
    state: tauri::State<'_, crate::AppState>,
    person_id: Option<u64>,
    name: Option<String>,
) -> Result<Value, String> {
    let s = crate::load_settings_inner(&state.kv);
    if s.tmdb_api_key.trim().is_empty() {
        return Err(
            "Configure a chave do TMDB em Ajustes → Discord, TMDB e MDBList para abrir páginas de elenco."
                .into(),
        );
    }
    let client = TmdbClient::new(s.tmdb_api_key, s.services.tmdb_language)
        .map_err(|e| e.to_string())?;
    let person_id = match person_id.filter(|id| *id > 0) {
        Some(id) => id,
        None => {
            let query = name.unwrap_or_default();
            if query.trim().is_empty() {
                return Err("Ator sem identificação.".into());
            }
            let found = client
                .search_person(query.trim())
                .await
                .map_err(|e| e.to_string())?;
            found["results"][0]["id"]
                .as_u64()
                .ok_or_else(|| "Ator não encontrado no TMDB.".to_string())?
        }
    };
    let data = client.person(person_id).await.map_err(|e| e.to_string())?;

    let known_for = data["known_for_department"].as_str().unwrap_or("").trim();
    let prefer_crew = !known_for.is_empty()
        && !matches!(known_for.to_ascii_lowercase().as_str(), "acting" | "actors");
    let cast_credits = map_person_credits(data["combined_credits"]["cast"].as_array(), false);
    let crew_credits = map_person_credits(data["combined_credits"]["crew"].as_array(), true);
    let mut credits = if (prefer_crew && !crew_credits.is_empty()) || cast_credits.is_empty() {
        crew_credits
    } else {
        cast_credits
    };
    credits.sort_by(|a, b| {
        b["rating"]
            .as_f64()
            .unwrap_or(0.0)
            .total_cmp(&a["rating"].as_f64().unwrap_or(0.0))
    });
    let credit_count = credits.len();

    Ok(json!({
        "id": person_id,
        "name": data["name"],
        "biography": data["biography"],
        "birthday": data["birthday"],
        "deathday": data["deathday"],
        "placeOfBirth": data["place_of_birth"],
        "knownFor": data["known_for_department"],
        "gender": data["gender"],
        "profile": image(&data["profile_path"], "w500"),
        "profileLarge": image(&data["profile_path"], "h632"),
        "homepage": data["homepage"],
        "creditCount": credit_count,
        "credits": credits,
    }))
}

fn localize_crew_job(job: &str) -> String {
    match job.trim().to_ascii_lowercase().as_str() {
        "director" => "Diretor".into(),
        "writer" | "screenplay" => "Roteirista".into(),
        "producer" => "Produtor".into(),
        "executive producer" => "Produtor Executivo".into(),
        "creator" => "Criador".into(),
        _ => job.trim().to_string(),
    }
}

fn map_person_credits(rows: Option<&Vec<Value>>, is_crew: bool) -> Vec<Value> {
    let mut seen = std::collections::HashSet::new();
    rows.into_iter()
        .flatten()
        .filter_map(|c| {
            let media_type = c["media_type"].as_str()?;
            if media_type != "movie" && media_type != "tv" {
                return None;
            }
            let tmdb_id = c["id"].as_u64()?;
            if !seen.insert(tmdb_id) {
                return None;
            }
            let title = c["title"]
                .as_str()
                .or_else(|| c["name"].as_str())
                .filter(|t| !t.trim().is_empty())?;
            let poster = image(&c["poster_path"], "w342").or_else(|| image(&c["backdrop_path"], "w780"));
            poster.as_ref()?;
            let date = c["release_date"]
                .as_str()
                .or_else(|| c["first_air_date"].as_str())
                .unwrap_or("");
            let role = if is_crew {
                localize_crew_job(c["job"].as_str().or_else(|| c["department"].as_str()).unwrap_or(""))
            } else {
                c["character"].as_str().unwrap_or("").to_string()
            };
            Some(json!({
                "id": format!("tmdb:{tmdb_id}"),
                "tmdbId": tmdb_id,
                "type": if media_type == "movie" { "movie" } else { "series" },
                "name": title,
                "character": role,
                "poster": poster,
                "banner": image(&c["backdrop_path"], "w1280"),
                "rating": c["vote_average"],
                "year": date.get(0..4).unwrap_or(""),
            }))
        })
        .collect()
}

#[tauri::command]
pub async fn tmdb_season_episodes(
    state: tauri::State<'_, crate::AppState>,
    series: u64,
    season: u32,
) -> Result<Vec<Value>, String> {
    let s = crate::load_settings_inner(&state.kv);
    if !s.services.tmdb_enabled || !s.services.tmdb_episodes {
        return Ok(Vec::new());
    }
    let client = TmdbClient::new(s.tmdb_api_key, s.services.tmdb_language)
        .map_err(|_| "Falha ao configurar o TMDB")?;
    let data = client
        .season(series, season)
        .await
        .map_err(|_| "Temporada indisponível no TMDB")?;
    Ok(data["episodes"].as_array().into_iter().flatten().filter_map(|ep| Some(json!({
        "episode":ep["episode_number"].as_u64()?,"season":season,"title":ep["name"],"overview":ep["overview"],
        "released":ep["air_date"],"thumbnail":image(&ep["still_path"],"w500")
    }))).collect())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarSeriesInput {
    pub id: String,
    pub name: String,
    pub tmdb_id: Option<u64>,
    pub imdb_id: Option<String>,
    pub poster: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReconciledDates {
    primary: String,
    streaming_date: Option<String>,
    tv_date: Option<String>,
}

fn parse_ymd_days(date_str: &str) -> Option<i64> {
    let raw = date_str.trim().get(..10)?;
    let mut parts = raw.split('-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y_adj = y - i64::from(m <= 2);
    let era = y_adj.div_euclid(400);
    let yoe = y_adj.rem_euclid(400);
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

fn reconcile_episode_dates(
    tmdb_date: Option<&str>,
    tvmaze_date: Option<&str>,
    has_web_channel: bool,
    has_network: bool,
) -> Option<ReconciledDates> {
    let td = tmdb_date.map(str::trim).filter(|s| s.len() >= 10).map(|s| &s[..10]);
    let md = tvmaze_date.map(str::trim).filter(|s| s.len() >= 10).map(|s| &s[..10]);
    match (td, md) {
        (Some(td), Some(md)) => {
            if td == md {
                return Some(ReconciledDates {
                    primary: md.to_string(),
                    streaming_date: has_web_channel.then(|| md.to_string()),
                    tv_date: has_network.then(|| md.to_string()),
                });
            }
            let diff = parse_ymd_days(td)
                .zip(parse_ymd_days(md))
                .map(|(a, b)| (a - b).abs())
                .unwrap_or(999);
            if diff <= 14 {
                if has_web_channel {
                    if md < td {
                        // Streaming drops earlier than linear TV (e.g. Paramount+ Friday vs Showtime Sunday).
                        return Some(ReconciledDates {
                            primary: md.to_string(),
                            streaming_date: Some(md.to_string()),
                            tv_date: Some(td.to_string()),
                        });
                    }
                    return Some(ReconciledDates {
                        primary: td.to_string(),
                        streaming_date: Some(td.to_string()),
                        tv_date: Some(md.to_string()),
                    });
                }
                if td < md {
                    return Some(ReconciledDates {
                        primary: td.to_string(),
                        streaming_date: Some(td.to_string()),
                        tv_date: Some(md.to_string()),
                    });
                }
                return Some(ReconciledDates {
                    primary: md.to_string(),
                    streaming_date: None,
                    tv_date: has_network.then(|| md.to_string()),
                });
            }
            let primary = if has_web_channel { md } else { td };
            Some(ReconciledDates {
                primary: primary.to_string(),
                streaming_date: has_web_channel.then(|| md.to_string()),
                tv_date: if has_network || td != md {
                    Some(td.to_string())
                } else {
                    None
                },
            })
        }
        (None, Some(md)) => Some(ReconciledDates {
            primary: md.to_string(),
            streaming_date: has_web_channel.then(|| md.to_string()),
            tv_date: has_network.then(|| md.to_string()),
        }),
        (Some(td), None) => Some(ReconciledDates {
            primary: td.to_string(),
            streaming_date: None,
            tv_date: Some(td.to_string()),
        }),
        (None, None) => None,
    }
}

fn strip_html_tags(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut in_tag = false;
    for ch in raw.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn url_encode_query(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

fn tvmaze_semaphore() -> &'static tokio::sync::Semaphore {
    static SEM: std::sync::OnceLock<tokio::sync::Semaphore> = std::sync::OnceLock::new();
    SEM.get_or_init(|| tokio::sync::Semaphore::new(6))
}

async fn tvmaze_get_json(http: &reqwest::Client, url: &str) -> Option<Value> {
    for attempt in 0..2 {
        let permit = tvmaze_semaphore().acquire().await.ok()?;
        let Ok(resp) = http.get(url).send().await else {
            return None;
        };
        let status = resp.status();
        if status.as_u16() == 429 && attempt == 0 {
            drop(permit);
            tokio::time::sleep(std::time::Duration::from_millis(650)).await;
            continue;
        }
        if !status.is_success() {
            return None;
        }
        return resp.json::<Value>().await.ok();
    }
    None
}

async fn fetch_tvmaze_show_with_episodes(
    http: &reqwest::Client,
    imdb_id: Option<&str>,
    tvdb_id: Option<u64>,
    name: &str,
) -> Option<Value> {
    let mut resolved_id = if let Some(imdb) = imdb_id.filter(|s| s.starts_with("tt")) {
        let url = format!("https://api.tvmaze.com/lookup/shows?imdb={imdb}");
        tvmaze_get_json(http, &url)
            .await
            .and_then(|val| val["id"].as_u64())
    } else {
        None
    };

    if resolved_id.is_none()
        && let Some(tvdb) = tvdb_id.filter(|&v| v > 0)
    {
        let url = format!("https://api.tvmaze.com/lookup/shows?thetvdb={tvdb}");
        resolved_id = tvmaze_get_json(http, &url)
            .await
            .and_then(|val| val["id"].as_u64());
    }

    if let Some(show_id) = resolved_id {
        let url = format!("https://api.tvmaze.com/shows/{show_id}?embed=episodes");
        if let Some(val) = tvmaze_get_json(http, &url).await {
            return Some(val);
        }
    }

    let trimmed = name.trim();
    if !trimmed.is_empty() {
        let url = format!(
            "https://api.tvmaze.com/singlesearch/shows?q={}&embed=episodes",
            url_encode_query(trimmed)
        );
        if let Some(val) = tvmaze_get_json(http, &url).await {
            return Some(val);
        }
    }
    None
}

async fn fetch_tmdb_calendar_data(
    client: &TmdbClient,
    cache: Option<&CacheStore>,
    initial_tmdb_id: Option<u64>,
    initial_imdb: Option<&str>,
) -> (Option<u64>, Option<Value>, BTreeMap<(u64, u64), Value>) {
    let mut resolved_tid = initial_tmdb_id;
    if resolved_tid.is_none()
        && let Some(imdb) = initial_imdb.filter(|s| s.starts_with("tt"))
    {
        let map_key = format!("imdb_tmdb_tv:{imdb}");
        if let Some(store) = cache
            && let Ok(Some(raw)) = store.get(&map_key)
            && let Ok(parsed) = serde_json::from_slice::<u64>(&raw)
            && parsed > 0
        {
            resolved_tid = Some(parsed);
        }
        if resolved_tid.is_none()
            && let Ok(Some(found)) = client.find_by_imdb(imdb).await
            && found > 0
        {
            resolved_tid = Some(found);
            if let Some(store) = cache
                && let Ok(bytes) = serde_json::to_vec(&found)
            {
                let _ = store.set(&map_key, &bytes, Some(7 * 86400));
            }
        }
    }

    let Some(tid) = resolved_tid else {
        return (None, None, BTreeMap::new());
    };
    let Ok(data) = client.tv_with_externals(tid).await else {
        return (Some(tid), None, BTreeMap::new());
    };

    let mut seasons_to_fetch: Vec<u64> = Vec::new();
    for key in ["next_episode_to_air", "last_episode_to_air"] {
        if let Some(s) = data[key]["season_number"].as_u64().filter(|&s| s > 0)
            && !seasons_to_fetch.contains(&s)
        {
            seasons_to_fetch.push(s);
        }
    }
    let mut season_set = tokio::task::JoinSet::new();
    for season_no in seasons_to_fetch {
        let client_clone = client.clone();
        season_set.spawn(async move {
            (season_no, client_clone.season(tid, season_no as u32).await.ok())
        });
    }

    let mut tmdb_eps: BTreeMap<(u64, u64), Value> = BTreeMap::new();
    while let Some(res) = season_set.join_next().await {
        if let Ok((season_no, Some(season))) = res
            && let Some(episodes) = season["episodes"].as_array()
        {
            for ep in episodes {
                let Some(ep_no) = ep["episode_number"].as_u64() else {
                    continue;
                };
                let s_no = ep["season_number"].as_u64().unwrap_or(season_no);
                tmdb_eps.insert((s_no, ep_no), ep.clone());
            }
        }
    }
    if let Some(next_ep) = data["next_episode_to_air"].as_object()
        && let (Some(s_no), Some(ep_no)) = (
            next_ep.get("season_number").and_then(|v| v.as_u64()),
            next_ep.get("episode_number").and_then(|v| v.as_u64()),
        )
    {
        tmdb_eps
            .entry((s_no, ep_no))
            .or_insert_with(|| Value::Object(next_ep.clone()));
    }

    (Some(tid), Some(data), tmdb_eps)
}

async fn resolve_series_calendar_entries(
    tmdb_client: Option<&TmdbClient>,
    http: &reqwest::Client,
    cache: Option<&CacheStore>,
    input: &CalendarSeriesInput,
) -> (Vec<Value>, bool) {
    let initial_imdb = input
        .imdb_id
        .as_deref()
        .or_else(|| input.id.starts_with("tt").then_some(input.id.as_str()))
        .map(str::trim)
        .filter(|s| s.starts_with("tt"))
        .map(str::to_string);
    let initial_tmdb = input
        .tmdb_id
        .filter(|&v| v > 0)
        .or_else(|| input.id.strip_prefix("tmdb:").and_then(|raw| raw.parse::<u64>().ok()));

    let can_query_tvmaze_early = initial_imdb.is_some() || !input.name.trim().is_empty();
    let tmdb_fut = async {
        match tmdb_client {
            Some(client) => {
                fetch_tmdb_calendar_data(client, cache, initial_tmdb, initial_imdb.as_deref())
                    .await
            }
            None => (initial_tmdb, None, BTreeMap::new()),
        }
    };
    let tvmaze_early_fut = async {
        if can_query_tvmaze_early {
            fetch_tvmaze_show_with_episodes(http, initial_imdb.as_deref(), None, &input.name).await
        } else {
            None
        }
    };
    let ((tmdb_id, tmdb_data, tmdb_eps), early_tvmaze) = tokio::join!(tmdb_fut, tvmaze_early_fut);

    let mut imdb_id = initial_imdb.clone();
    let mut tvdb_id: Option<u64> = None;
    let mut show_name = input.name.clone();
    let mut poster: Option<String> = input.poster.clone().filter(|s| !s.is_empty());
    let mut backdrop: Option<String> = None;
    let mut tmdb_network: Option<String> = None;
    let mut is_ended = false;

    if let Some(ref data) = tmdb_data {
        if imdb_id.is_none() {
            imdb_id = data["external_ids"]["imdb_id"]
                .as_str()
                .map(str::trim)
                .filter(|s| s.starts_with("tt"))
                .map(str::to_string);
        }
        tvdb_id = data["external_ids"]["tvdb_id"].as_u64();
        if let Some(n) = data["name"].as_str().map(str::trim).filter(|s| !s.is_empty()) {
            show_name = n.to_string();
        }
        if let Some(p) = image(&data["poster_path"], "w342") {
            poster = Some(p);
        }
        backdrop = image(&data["backdrop_path"], "w780");
        tmdb_network = data["networks"][0]["name"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        is_ended = matches!(
            data["status"].as_str().map(str::trim),
            Some("Ended" | "Canceled")
        );
    }

    // Fallback to TVMaze if earlier lookup was skipped (`tmdb:...` id with empty name) or lacked IDs.
    let tvmaze = if early_tvmaze.is_some() {
        early_tvmaze
    } else if !can_query_tvmaze_early
        || (initial_imdb.is_none() && (imdb_id.is_some() || tvdb_id.is_some()))
    {
        fetch_tvmaze_show_with_episodes(http, imdb_id.as_deref(), tvdb_id, &show_name).await
    } else {
        None
    };

    let mut web_channel: Option<String> = None;
    let mut tv_network: Option<String> = tmdb_network;
    let mut schedule_days: Vec<String> = Vec::new();
    let mut tvmaze_eps: BTreeMap<(u64, u64), Value> = BTreeMap::new();

    if let Some(ref show) = tvmaze {
        if show["status"].as_str() == Some("Ended") {
            is_ended = true;
        }
        if show_name.is_empty()
            && let Some(n) = show["name"].as_str().map(str::trim).filter(|s| !s.is_empty())
        {
            show_name = n.to_string();
        }
        if poster.is_none() {
            poster = show["image"]["medium"]
                .as_str()
                .or_else(|| show["image"]["original"].as_str())
                .map(str::to_string);
        }
        if imdb_id.is_none() {
            imdb_id = show["externals"]["imdb"]
                .as_str()
                .map(str::trim)
                .filter(|s| s.starts_with("tt"))
                .map(str::to_string);
        }
        web_channel = show["webChannel"]["name"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        if let Some(net) = show["network"]["name"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            tv_network = Some(net.to_string());
        }
        if let Some(days) = show["schedule"]["days"].as_array() {
            schedule_days = days
                .iter()
                .filter_map(|d| d.as_str().map(str::trim).filter(|s| !s.is_empty()))
                .map(str::to_string)
                .collect();
        }
        if let Some(episodes) = show["_embedded"]["episodes"].as_array() {
            for ep in episodes {
                let Some(s_no) = ep["season"].as_u64() else {
                    continue;
                };
                let Some(ep_no) = ep["number"].as_u64() else {
                    continue;
                };
                if s_no == 0 {
                    continue;
                }
                tvmaze_eps.insert((s_no, ep_no), ep.clone());
            }
        }
    }

    let mut keys: BTreeMap<(u64, u64), ()> = BTreeMap::new();
    for &k in tmdb_eps.keys() {
        keys.insert(k, ());
    }
    for &k in tvmaze_eps.keys() {
        keys.insert(k, ());
    }

    let has_web = web_channel.is_some();
    let has_net = tv_network.is_some();
    let mut out = Vec::new();
    for (season_no, ep_no) in keys.keys().copied() {
        let tm = tmdb_eps.get(&(season_no, ep_no));
        let mz = tvmaze_eps.get(&(season_no, ep_no));
        let tmdb_date = tm.and_then(|v| v["air_date"].as_str());
        let tvmaze_date = mz.and_then(|v| v["airdate"].as_str());

        let reconciled = reconcile_episode_dates(tmdb_date, tvmaze_date, has_web, has_net);
        let title = tm
            .and_then(|v| v["name"].as_str().map(str::trim).filter(|s| !s.is_empty()))
            .or_else(|| mz.and_then(|v| v["name"].as_str().map(str::trim).filter(|s| !s.is_empty())))
            .unwrap_or("")
            .to_string();
        let tmdb_overview = tm
            .and_then(|v| v["overview"].as_str().map(str::trim).filter(|s| !s.is_empty()))
            .map(str::to_string);
        let overview = tmdb_overview.unwrap_or_else(|| {
            mz.and_then(|v| v["summary"].as_str())
                .map(strip_html_tags)
                .unwrap_or_default()
        });
        let ep_thumb = tm
            .and_then(|v| image(&v["still_path"], "w500"))
            .or_else(|| {
                mz.and_then(|v| {
                    v["image"]["original"]
                        .as_str()
                        .or_else(|| v["image"]["medium"].as_str())
                        .map(str::to_string)
                })
            });
        let airstamp = mz
            .and_then(|v| v["airstamp"].as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);

        match reconciled {
            Some(rec) => {
                out.push(json!({
                    "id": &input.id,
                    "tmdb_id": tmdb_id.unwrap_or(0),
                    "imdb_id": &imdb_id,
                    "name": &show_name,
                    "season": season_no,
                    "episode": ep_no,
                    "title": title,
                    "overview": overview,
                    "released": rec.primary,
                    "streaming_date": rec.streaming_date,
                    "tv_date": rec.tv_date,
                    "airstamp": airstamp,
                    "platform": &web_channel,
                    "network": &tv_network,
                    "schedule_days": &schedule_days,
                    "poster": &poster,
                    "backdrop": ep_thumb.as_ref().or(backdrop.as_ref()),
                }));
            }
            None => {
                // Only emit undated episodes for the active/upcoming season from TMDB.
                if tm.is_some() {
                    out.push(json!({
                        "id": &input.id,
                        "tmdb_id": tmdb_id.unwrap_or(0),
                        "imdb_id": &imdb_id,
                        "name": &show_name,
                        "season": season_no,
                        "episode": ep_no,
                        "title": title,
                        "overview": overview,
                        "released": "",
                        "undated": true,
                        "platform": &web_channel,
                        "network": &tv_network,
                        "schedule_days": &schedule_days,
                        "poster": &poster,
                        "backdrop": backdrop.as_ref(),
                    }));
                }
            }
        }
    }
    (out, is_ended)
}

/// Official release calendar for library series, reconciling streaming platforms (TVMaze)
/// with TV/localized metadata (TMDB) so hybrid releases (e.g. Paramount+ Friday vs Showtime Sunday)
/// appear on their true official release day.
#[tauri::command]
pub async fn calendar_episodes(
    state: tauri::State<'_, crate::AppState>,
    series: Vec<CalendarSeriesInput>,
    from: Option<String>,
    force: Option<bool>,
) -> Result<Vec<Value>, String> {
    const CALENDAR_ACTIVE_CACHE_TTL_SECS: u64 = 6 * 3600;
    const CALENDAR_ENDED_CACHE_TTL_SECS: u64 = 24 * 3600;
    let s = crate::load_settings_inner(&state.kv);
    let tmdb_client = if s.services.tmdb_enabled && !s.tmdb_api_key.trim().is_empty() {
        TmdbClient::new(s.tmdb_api_key, s.services.tmdb_language).ok()
    } else {
        None
    };
    let http = shared_http_client().map_err(|e| e.to_string())?;
    let lower_bound = from.unwrap_or_default().trim().to_string();
    let force = force.unwrap_or(false);

    let mut unique: Vec<CalendarSeriesInput> = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();
    for item in series.into_iter().take(120) {
        let key = item.id.trim().to_string();
        if key.is_empty() || !seen_ids.insert(key) {
            continue;
        }
        unique.push(item);
    }

    let mut pending = unique.into_iter();
    let mut tasks: tokio::task::JoinSet<Vec<Value>> = tokio::task::JoinSet::new();
    let cache = state.cache.clone();
    let spawn = |tasks: &mut tokio::task::JoinSet<Vec<Value>>, input: CalendarSeriesInput| {
        let tmdb_client = tmdb_client.clone();
        let http = http.clone();
        let cache = cache.clone();
        let tmdb_active = tmdb_client.is_some();
        tasks.spawn(async move {
            let cache_key = format!(
                "calendar_series_v3:{}:{}:{}",
                input.id,
                input.tmdb_id.unwrap_or(0),
                tmdb_active
            );
            if !force
                && let Ok(Some(raw)) = cache.get(&cache_key)
                && let Ok(cached) = serde_json::from_slice::<Vec<Value>>(&raw)
            {
                return cached;
            }
            let (resolved, is_ended) = resolve_series_calendar_entries(
                tmdb_client.as_ref(),
                &http,
                Some(&cache),
                &input,
            )
            .await;
            if !resolved.is_empty()
                && let Ok(bytes) = serde_json::to_vec(&resolved)
            {
                let ttl = if is_ended {
                    CALENDAR_ENDED_CACHE_TTL_SECS
                } else {
                    CALENDAR_ACTIVE_CACHE_TTL_SECS
                };
                let _ = cache.set(&cache_key, &bytes, Some(ttl));
            }
            resolved
        });
    };

    for _ in 0..12 {
        match pending.next() {
            Some(item) => spawn(&mut tasks, item),
            None => break,
        }
    }

    let mut out = Vec::new();
    while let Some(res) = tasks.join_next().await {
        if let Ok(values) = res {
            for ep in values {
                let released = ep["released"].as_str().unwrap_or("");
                let undated = ep["undated"].as_bool().unwrap_or(false);
                if !undated && !lower_bound.is_empty() && released < lower_bound.as_str() {
                    continue;
                }
                out.push(ep);
            }
        }
        if let Some(item) = pending.next() {
            spawn(&mut tasks, item);
        }
    }

    out.sort_by(|a, b| {
        calendar_sort_key(a)
            .cmp(calendar_sort_key(b))
            .then_with(|| a["name"].as_str().unwrap_or("").cmp(b["name"].as_str().unwrap_or("")))
            .then_with(|| a["season"].as_u64().cmp(&b["season"].as_u64()))
            .then_with(|| a["episode"].as_u64().cmp(&b["episode"].as_u64()))
    });
    Ok(out)
}

/// Upcoming episodes of every library series, for the release calendar.
///
/// `next_episode_to_air` only carries a single episode, so the season of that
/// episode is fetched as well to return every remaining date. Both requests use
/// the slim endpoints (no credits/images/videos) and run in parallel; `from`
/// (YYYY-MM-DD) trims past episodes before they cross the IPC boundary.
#[tauri::command]
pub async fn tmdb_next_episodes(
    state: tauri::State<'_, crate::AppState>,
    series: Vec<u64>,
    from: Option<String>,
) -> Result<Vec<Value>, String> {
    let s = crate::load_settings_inner(&state.kv);
    if !s.services.tmdb_enabled || s.tmdb_api_key.trim().is_empty() {
        return Ok(Vec::new());
    }
    let client = TmdbClient::new(s.tmdb_api_key, s.services.tmdb_language)
        .map_err(|_| "Falha ao configurar o TMDB")?;
    let lower_bound = from.unwrap_or_default().trim().to_string();
    let mut ids: Vec<u64> = series.into_iter().take(120).collect();
    ids.sort_unstable();
    ids.dedup();
    let mut pending = ids.into_iter();
    let mut tasks: tokio::task::JoinSet<Vec<Value>> = tokio::task::JoinSet::new();
    let spawn = |tasks: &mut tokio::task::JoinSet<Vec<Value>>, id: u64| {
        let client = client.clone();
        let from = lower_bound.clone();
        tasks.spawn(async move {
            let mut out = Vec::new();
            let Ok(data) = client.tv(id).await else {
                return out;
            };
            let next = &data["next_episode_to_air"];
            let Some(season_no) = next["season_number"].as_u64() else {
                return out;
            };
            let next_date = next["air_date"].as_str().unwrap_or("");
            let next_episode = next["episode_number"].as_u64().unwrap_or(0);
            if next_date.is_empty() {
                return out;
            }
            let mut has_dated = false;
            let poster = image(&data["poster_path"], "w342");
            let backdrop = image(&data["backdrop_path"], "w780");
            let name = data["name"].clone();
            if let Ok(season) = client.season(id, season_no as u32).await
                && let Some(episodes) = season["episodes"].as_array()
            {
                for ep in episodes {
                    let released = ep["air_date"].as_str().unwrap_or("");
                    if released.is_empty() {
                        // Episódio já listado no TMDB mas ainda sem data: entra na
                        // agenda como "data a anunciar" em vez de desaparecer.
                        if ep["episode_number"].as_u64().unwrap_or(0) > next_episode {
                            out.push(json!({
                                "tmdb_id": id,
                                "name": &name,
                                "season": ep["season_number"].as_u64().or(Some(season_no)),
                                "episode": ep["episode_number"],
                                "title": ep["name"],
                                "overview": ep["overview"],
                                "released": "",
                                "undated": true,
                                "poster": &poster,
                                "backdrop": &backdrop,
                            }));
                        }
                        continue;
                    }
                    if !from.is_empty() && released < from.as_str() {
                        continue;
                    }
                    has_dated = true;
                    out.push(json!({
                        "tmdb_id": id,
                        "name": &name,
                        "season": ep["season_number"].as_u64().or(Some(season_no)),
                        "episode": ep["episode_number"],
                        "title": ep["name"],
                        "overview": ep["overview"],
                        "released": released,
                        "poster": &poster,
                        "backdrop": &backdrop,
                    }));
                }
            }
            if !has_dated {
                // Temporada ainda não publicada no TMDB: mantém o episódio anunciado.
                out.push(json!({
                    "tmdb_id": id,
                    "name": &name,
                    "season": season_no,
                    "episode": next["episode_number"],
                    "title": next["name"],
                    "overview": next["overview"],
                    "released": next_date,
                    "poster": &poster,
                    "backdrop": &backdrop,
                }));
            }
            out
        });
    };
    for _ in 0..10 {
        match pending.next() {
            Some(id) => spawn(&mut tasks, id),
            None => break,
        }
    }
    let mut out = Vec::new();
    while let Some(res) = tasks.join_next().await {
        if let Ok(mut values) = res {
            out.append(&mut values);
        }
        if let Some(id) = pending.next() {
            spawn(&mut tasks, id);
        }
    }
    out.sort_by(|a, b| {
        calendar_sort_key(a).cmp(calendar_sort_key(b)).then_with(|| {
            a["name"]
                .as_str()
                .unwrap_or("")
                .cmp(b["name"].as_str().unwrap_or(""))
        })
    });
    Ok(out)
}

/// Dated episodes first (oldest) and "data a anunciar" last.
fn calendar_sort_key(episode: &Value) -> &str {
    match episode["released"].as_str() {
        Some(released) if !released.is_empty() => released,
        _ => "9999-12-31",
    }
}

#[tauri::command]
pub async fn hero_ratings(
    state: tauri::State<'_, crate::AppState>,
    id: String,
    kind: String,
) -> Result<Value, String> {
    let s = crate::load_settings_inner(&state.kv).services;
    let lookup = if id
        .strip_prefix("tt")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
    {
        Some(("imdb", id.as_str()))
    } else {
        id.strip_prefix("tmdb:")
            .filter(|n| n.parse::<u64>().is_ok())
            .map(|n| ("tmdb", n))
    };
    if s.mdblist_enabled
        && s.mdblist_hero
        && !s.providers.is_empty()
        && let Some((provider, id)) = lookup
    {
        let client =
            MdbListClient::new(&s.mdblist_key).map_err(|_| "Falha ao configurar o MDBList")?;
        let rows = client
            .ratings(provider, &kind, id)
            .await
            .map_err(|_| "Notas indisponíveis")?;
        return Ok(s.ratings(&rows));
    }
    Ok(json!([]))
}

fn person_photo_cache() -> &'static std::sync::Mutex<std::collections::HashMap<String, Value>> {
    static CACHE: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, Value>>,
    > = std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

#[tauri::command]
pub async fn resolve_people_photos(
    state: tauri::State<'_, crate::AppState>,
    names: Vec<String>,
) -> Result<std::collections::HashMap<String, Value>, String> {
    let s = crate::load_settings_inner(&state.kv);
    if s.tmdb_api_key.trim().is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let client = TmdbClient::new(s.tmdb_api_key, s.services.tmdb_language)
        .map_err(|e| e.to_string())?;
    let mut out = std::collections::HashMap::new();
    let mut pending = Vec::new();
    {
        let cache = person_photo_cache().lock().expect("photo cache");
        for raw in names.into_iter().take(18) {
            let trimmed = raw.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }
            let key = trimmed.to_lowercase();
            if let Some(hit) = cache.get(&key) {
                out.insert(trimmed, hit.clone());
            } else if !pending.iter().any(|p: &String| p.eq_ignore_ascii_case(&trimmed)) {
                pending.push(trimmed);
            }
        }
    }
    for name in pending {
        let Ok(found) = client.search_person(&name).await else {
            continue;
        };
        let Some(first) = found["results"].get(0) else {
            continue;
        };
        let entry = json!({
            "id": first["id"],
            "photo": image(&first["profile_path"], "w185"),
        });
        person_photo_cache()
            .lock()
            .expect("photo cache")
            .insert(name.to_lowercase(), entry.clone());
        out.insert(name, entry);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enrichment_preserves_disabled_fields_and_addon_episodes() {
        let prefs = Services {
            tmdb_text: false,
            tmdb_images: false,
            ..Default::default()
        };
        let mut meta = json!({"name":"Original","poster":"addon","videos":[{"id":"addon:1"}]});
        prefs.merge_tmdb(
            &mut meta,
            &json!({
                "name":"TMDB","poster_path":"/other","genres":[{"name":"Anime"}],
                "seasons":[{"season_number":1,"poster_path":"/season.jpg"}],
                "recommendations":{"results":[{"id":21,"name":"Outra série"}]}
            }),
            "series",
        );
        assert_eq!(meta["name"], "Original");
        assert_eq!(meta["poster"], "addon");
        assert_eq!(meta["videos"][0]["id"], "addon:1");
        assert_eq!(meta["genres"][0], "Anime");
        assert_eq!(meta["tmdbSimilar"][0]["id"], "tmdb:21");
        assert_eq!(meta["tmdbSeasons"][0]["number"], 1);
    }

    #[test]
    fn rating_units_aliases_and_disabled_sources() {
        let prefs = Services {
            providers: vec!["imdb".into(), "letterboxd".into(), "audience".into()],
            ..Default::default()
        };
        let result = prefs.ratings(&[
            json!({"source":"imdb","value":8.3}),
            json!({"source":"letterboxd","score":90}),
            json!({"source":"popcorn","value":80}),
            json!({"source":"audience","score":80}),
            json!({"source":"mal","value":8}),
        ]);
        assert_eq!(result.as_array().unwrap().len(), 3);
        assert_eq!(result[0]["maximum"], 10.0);
        assert_eq!(result[1]["value"], 4.5);
        assert_eq!(result[2]["source"], "audience");
    }

    #[test]
    fn enrichment_includes_director_first_productions_trailers_cert_and_crew() {
        let prefs = Services::default();
        let mut meta = json!({"name": "Jogos Vorazes"});
        prefs.merge_tmdb(
            &mut meta,
            &json!({
                "title": "Jogos Vorazes",
                "status": "Released",
                "original_language": "en",
                "runtime": 142,
                "production_countries": [{"iso_3166_1": "US", "name": "United States"}],
                "production_companies": [
                    {"id": 1632, "name": "Lionsgate", "logo_path": "/cisLn1YAUuptXVBa0xjq7ST9cH0.png"}
                ],
                "release_dates": {
                    "results": [
                        {"iso_3166_1": "US", "release_dates": [{"certification": "PG-13"}]},
                        {"iso_3166_1": "BR", "release_dates": [{"certification": "14"}]}
                    ]
                },
                "credits": {
                    "crew": [{"id": 18357, "name": "Gary Ross", "job": "Director", "profile_path": "/dir.jpg"}],
                    "cast": [{"id": 72129, "name": "Jennifer Lawrence", "character": "Katniss Everdeen", "profile_path": "/kat.jpg"}]
                },
                "videos": {
                    "results": [
                        {"site": "YouTube", "type": "Teaser", "key": "abc", "name": "Teaser Oficial", "published_at": "2011-08-01T00:00:00Z"},
                        {"site": "YouTube", "type": "Trailer", "key": "xyz", "name": "Trailer - Jogos Vorazes", "published_at": "2011-11-14T00:00:00Z"}
                    ]
                }
            }),
            "movie",
        );

        assert_eq!(meta["country"], "US");
        assert_eq!(meta["ageRating"], "14");
        assert_eq!(meta["castMembers"][0]["name"], "Gary Ross");
        assert_eq!(meta["castMembers"][0]["character"], "Diretor");
        assert_eq!(meta["castMembers"][1]["name"], "Jennifer Lawrence");
        assert_eq!(meta["productionCompanies"][0]["name"], "Lionsgate");
        assert!(meta["productionCompanies"][0]["logo"].as_str().unwrap().contains("w300"));
        assert_eq!(meta["trailers"][0]["key"], "xyz");
        assert_eq!(meta["trailers"][0]["name"], "Trailer - Jogos Vorazes");

        let crew = map_person_credits(
            Some(&vec![json!({
                "id": 70160,
                "media_type": "movie",
                "title": "Jogos Vorazes",
                "job": "Director",
                "poster_path": "/poster.jpg",
                "vote_average": 7.2,
                "release_date": "2012-03-12"
            })]),
            true,
        );
        assert_eq!(crew.len(), 1);
        assert_eq!(crew[0]["character"], "Diretor");

        // Collections stay chronologically ordered and keep the active movie in the franchise rail.
        let ordered = collection_cards(&json!([
            {"id": 102, "title": "A Esperança - Parte 1", "release_date": "2014-11-20", "poster_path": "/p3.jpg"},
            {"id": 100, "title": "Jogos Vorazes", "release_date": "2012-03-12", "poster_path": "/p1.jpg", "backdrop_path": "/b1.jpg", "vote_average": 7.21},
            {"id": 101, "title": "Em Chamas", "release_date": "2013-11-15", "poster_path": "/p2.jpg"}
        ]));
        let arr = ordered.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["name"], "Jogos Vorazes");
        assert_eq!(arr[0]["release_info"], "2012");
        assert_eq!(arr[1]["name"], "Em Chamas");
        assert_eq!(arr[2]["name"], "A Esperança - Parte 1");
    }

    #[test]
    fn reconciles_dexter_streaming_friday_over_linear_sunday() {
        // Paramount+ drops Friday (2026-11-27), Showtime linear TV airs Sunday (2026-11-29).
        let rec = reconcile_episode_dates(Some("2026-11-29"), Some("2026-11-27"), true, false)
            .expect("reconciled");
        assert_eq!(rec.primary, "2026-11-27");
        assert_eq!(rec.streaming_date.as_deref(), Some("2026-11-27"));
        assert_eq!(rec.tv_date.as_deref(), Some("2026-11-29"));

        // Matching dates preserve platform vs network distinction.
        let same = reconcile_episode_dates(Some("2026-12-04"), Some("2026-12-04"), true, true)
            .expect("same");
        assert_eq!(same.primary, "2026-12-04");
        assert_eq!(same.streaming_date.as_deref(), Some("2026-12-04"));
        assert_eq!(same.tv_date.as_deref(), Some("2026-12-04"));

        // TVMaze-only fallback works when TMDB is disabled.
        let mz_only = reconcile_episode_dates(None, Some("2026-12-11"), true, false)
            .expect("tvmaze only");
        assert_eq!(mz_only.primary, "2026-12-11");
        assert_eq!(mz_only.streaming_date.as_deref(), Some("2026-12-11"));
        assert_eq!(mz_only.tv_date, None);
    }
}
