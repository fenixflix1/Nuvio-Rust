//! Configurable in-app DNS resolver (DoH + UDP DNS) with Cloudflare (`1.1.1.1`) as the default.

use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::{Arc, LazyLock, RwLock};
use std::time::{Duration, Instant};
use tokio::net::{UdpSocket, lookup_host};

const CACHE_TTL: Duration = Duration::from_secs(300);
const QUERY_TIMEOUT: Duration = Duration::from_secs(4);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsConfig {
    pub mode: String,
    pub custom: String,
}

impl Default for DnsConfig {
    fn default() -> Self {
        Self {
            mode: "cloudflare".to_string(),
            custom: String::new(),
        }
    }
}

#[derive(Clone)]
struct CachedAddrs {
    ips: Vec<IpAddr>,
    expires_at: Instant,
}

static CONFIG: LazyLock<RwLock<DnsConfig>> = LazyLock::new(|| RwLock::new(DnsConfig::default()));
static CACHE: LazyLock<RwLock<HashMap<String, CachedAddrs>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
static RESOLVER: LazyLock<Arc<NuvioDnsResolver>> =
    LazyLock::new(|| Arc::new(NuvioDnsResolver));

// Bootstrap clients use system/pinned endpoints without calling NuvioDnsResolver to avoid recursion.
static BOOTSTRAP_PINNED: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(QUERY_TIMEOUT)
        .no_proxy()
        .resolve("dns.quad9.net", SocketAddr::new(IpAddr::V4(Ipv4Addr::new(9, 9, 9, 9)), 443))
        .resolve("dns.adguard-dns.com", SocketAddr::new(IpAddr::V4(Ipv4Addr::new(94, 140, 14, 14)), 443))
        .resolve("dns.google", SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 443))
        .build()
        .unwrap_or_default()
});

static BOOTSTRAP_PLAIN: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(QUERY_TIMEOUT)
        .no_proxy()
        .build()
        .unwrap_or_default()
});

/// Updates the active DNS provider and clears cached entries so changes apply immediately.
pub fn configure(mode: &str, custom: &str) {
    let norm_mode = normalize_mode(mode);
    let norm_custom = custom.trim().to_string();
    let mut cfg = CONFIG.write().expect("dns config lock");
    if cfg.mode != norm_mode || cfg.custom != norm_custom {
        cfg.mode = norm_mode;
        cfg.custom = norm_custom;
        CACHE.write().expect("dns cache lock").clear();
    }
}

pub fn normalize_mode(mode: &str) -> String {
    match mode.trim().to_ascii_lowercase().as_str() {
        "google" | "quad9" | "adguard" | "custom" | "system" => mode.trim().to_ascii_lowercase(),
        _ => "cloudflare".to_string(),
    }
}

pub fn resolver() -> Arc<NuvioDnsResolver> {
    Arc::clone(&RESOLVER)
}

pub fn client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder().dns_resolver(resolver())
}

#[derive(Debug)]
pub struct NuvioDnsResolver;

impl Resolve for NuvioDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_string();
        Box::pin(async move {
            let ips = resolve_host_ips(&host).await?;
            let addrs: Addrs = Box::new(ips.into_iter().map(|ip| SocketAddr::new(ip, 0)));
            Ok(addrs)
        })
    }
}

async fn resolve_host_ips(
    host: &str,
) -> std::result::Result<Vec<IpAddr>, Box<dyn std::error::Error + Send + Sync>> {
    let clean = host.trim().trim_end_matches('.');
    if let Ok(ip) = IpAddr::from_str(clean) {
        return Ok(vec![ip]);
    }
    if clean.eq_ignore_ascii_case("localhost") || clean.ends_with(".local") {
        return system_lookup(clean).await;
    }

    let cfg = CONFIG.read().expect("dns config lock").clone();
    if cfg.mode == "system" {
        return system_lookup(clean).await;
    }

    let cache_key = format!("{}|{}|{}", cfg.mode, cfg.custom, clean.to_ascii_lowercase());
    if let Some(hit) = CACHE.read().expect("dns cache lock").get(&cache_key).cloned()
        && Instant::now() < hit.expires_at
        && !hit.ips.is_empty()
    {
        return Ok(hit.ips);
    }

    let resolved = match cfg.mode.as_str() {
        "google" => doh_or_udp("https://dns.google/dns-query", IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), clean).await,
        "quad9" => doh_or_udp("https://dns.quad9.net/dns-query", IpAddr::V4(Ipv4Addr::new(9, 9, 9, 9)), clean).await,
        "adguard" => doh_or_udp("https://dns.adguard-dns.com/dns-query", IpAddr::V4(Ipv4Addr::new(94, 140, 14, 14)), clean).await,
        "custom" => resolve_custom(&cfg.custom, clean).await,
        _ => doh_or_udp("https://1.1.1.1/dns-query", IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), clean).await,
    };

    let ips = match resolved {
        Ok(list) if !list.is_empty() => list,
        _ => return system_lookup(clean).await,
    };

    CACHE.write().expect("dns cache lock").insert(
        cache_key,
        CachedAddrs {
            ips: ips.clone(),
            expires_at: Instant::now() + CACHE_TTL,
        },
    );
    Ok(ips)
}

async fn resolve_custom(custom: &str, host: &str) -> std::result::Result<Vec<IpAddr>, ()> {
    let trimmed = custom.trim();
    if trimmed.is_empty() {
        return doh_or_udp("https://1.1.1.1/dns-query", IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), host).await;
    }
    if let Ok(ip) = IpAddr::from_str(trimmed) {
        return udp_dns_query(SocketAddr::new(ip, 53), host).await;
    }
    if let Ok(addr) = SocketAddr::from_str(trimmed) {
        return udp_dns_query(addr, host).await;
    }
    if trimmed.starts_with("https://") {
        return doh_wire_query(&BOOTSTRAP_PLAIN, trimmed, host).await;
    }
    Err(())
}

async fn doh_or_udp(doh_url: &str, udp_ip: IpAddr, host: &str) -> std::result::Result<Vec<IpAddr>, ()> {
    if let Ok(ips) = doh_wire_query(&BOOTSTRAP_PINNED, doh_url, host).await
        && !ips.is_empty()
    {
        return Ok(ips);
    }
    udp_dns_query(SocketAddr::new(udp_ip, 53), host).await
}

async fn doh_wire_query(client: &reqwest::Client, url: &str, host: &str) -> std::result::Result<Vec<IpAddr>, ()> {
    let query = build_udp_query(host).ok_or(())?;
    let resp = client
        .post(url)
        .header("Content-Type", "application/dns-message")
        .header("Accept", "application/dns-message")
        .body(query)
        .send()
        .await
        .map_err(|_| ())?;
    if !resp.status().is_success() {
        return Err(());
    }
    let bytes = resp.bytes().await.map_err(|_| ())?;
    let ips = parse_udp_a_records(&bytes);
    if ips.is_empty() { Err(()) } else { Ok(ips) }
}

async fn udp_dns_query(server: SocketAddr, host: &str) -> std::result::Result<Vec<IpAddr>, ()> {
    let query = build_udp_query(host).ok_or(())?;
    let bind_addr = if server.is_ipv6() { "[::]:0" } else { "0.0.0.0:0" };
    let sock = UdpSocket::bind(bind_addr).await.map_err(|_| ())?;
    sock.connect(server).await.map_err(|_| ())?;
    sock.send(&query).await.map_err(|_| ())?;
    let mut buf = [0u8; 512];
    let n = tokio::time::timeout(QUERY_TIMEOUT, sock.recv(&mut buf))
        .await
        .map_err(|_| ())?
        .map_err(|_| ())?;
    let ips = parse_udp_a_records(&buf[..n]);
    if ips.is_empty() { Err(()) } else { Ok(ips) }
}

async fn system_lookup(
    host: &str,
) -> std::result::Result<Vec<IpAddr>, Box<dyn std::error::Error + Send + Sync>> {
    let addrs: Vec<IpAddr> = lookup_host((host, 0)).await?.map(|a| a.ip()).collect();
    if addrs.is_empty() {
        return Err(format!("No addresses resolved for {host}").into());
    }
    Ok(addrs)
}

/// Builds a minimal RFC 1035 DNS A-record query packet.
pub fn build_udp_query(host: &str) -> Option<Vec<u8>> {
    let clean = host.trim().trim_end_matches('.');
    if clean.is_empty() || clean.len() > 253 {
        return None;
    }
    let mut pkt = Vec::with_capacity(32 + clean.len());
    // Header: ID=0x1337, RD=1, QDCOUNT=1
    pkt.extend_from_slice(&[0x13, 0x37, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    for label in clean.split('.') {
        if label.is_empty() || label.len() > 63 {
            return None;
        }
        pkt.push(label.len() as u8);
        pkt.extend_from_slice(label.as_bytes());
    }
    pkt.push(0); // Root label
    pkt.extend_from_slice(&[0x00, 0x01, 0x00, 0x01]); // QTYPE=A, QCLASS=IN
    Some(pkt)
}

fn skip_dns_name(pkt: &[u8], mut idx: usize) -> Option<usize> {
    let mut hops = 0;
    while idx < pkt.len() {
        let len = pkt[idx];
        if len == 0 {
            return Some(idx + 1);
        }
        if (len & 0xC0) == 0xC0 {
            return if idx + 1 < pkt.len() { Some(idx + 2) } else { None };
        }
        idx += 1 + (len as usize);
        hops += 1;
        if hops > 64 {
            return None;
        }
    }
    None
}

/// Extracts IPv4 A-record addresses from an RFC 1035 DNS wire response.
pub fn parse_udp_a_records(pkt: &[u8]) -> Vec<IpAddr> {
    if pkt.len() < 12 {
        return Vec::new();
    }
    let qdcount = u16::from_be_bytes([pkt[4], pkt[5]]) as usize;
    let ancount = u16::from_be_bytes([pkt[6], pkt[7]]) as usize;
    let mut idx = 12;
    for _ in 0..qdcount {
        let Some(next) = skip_dns_name(pkt, idx) else {
            return Vec::new();
        };
        idx = next + 4; // skip QTYPE + QCLASS
        if idx > pkt.len() {
            return Vec::new();
        }
    }
    let mut out = Vec::new();
    for _ in 0..ancount {
        let Some(next) = skip_dns_name(pkt, idx) else {
            break;
        };
        idx = next;
        if idx + 10 > pkt.len() {
            break;
        }
        let rtype = u16::from_be_bytes([pkt[idx], pkt[idx + 1]]);
        let rclass = u16::from_be_bytes([pkt[idx + 2], pkt[idx + 3]]);
        let rdlength = u16::from_be_bytes([pkt[idx + 8], pkt[idx + 9]]) as usize;
        idx += 10;
        if idx + rdlength > pkt.len() {
            break;
        }
        if rtype == 1 && rclass == 1 && rdlength == 4 {
            out.push(IpAddr::V4(Ipv4Addr::new(
                pkt[idx],
                pkt[idx + 1],
                pkt[idx + 2],
                pkt[idx + 3],
            )));
        }
        idx += rdlength;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_and_parses_dns_a_packet() {
        let q = build_udp_query("api.nuvio.tv").expect("valid query");
        assert_eq!(&q[12..16], &[3, b'a', b'p', b'i']);

        let mut resp = q.clone();
        resp[2] = 0x81;
        resp[3] = 0x80;
        resp[7] = 1; // ANCOUNT = 1
        // Compressed pointer to offset 12, TYPE=A, CLASS=IN, TTL=60, RDLEN=4, IP=1.1.1.1
        resp.extend_from_slice(&[
            0xC0, 0x0C, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3C, 0x00, 0x04, 1, 1, 1, 1,
        ]);
        assert_eq!(
            parse_udp_a_records(&resp),
            vec![IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))]
        );
    }

    #[test]
    fn normalizes_dns_mode_with_cloudflare_default() {
        assert_eq!(normalize_mode(""), "cloudflare");
        assert_eq!(normalize_mode("CLOUDFLARE"), "cloudflare");
        assert_eq!(normalize_mode("google"), "google");
        assert_eq!(normalize_mode("custom"), "custom");
        assert_eq!(normalize_mode("system"), "system");
    }
}
