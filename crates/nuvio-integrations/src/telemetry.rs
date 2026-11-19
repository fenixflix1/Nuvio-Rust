/// Initializes Sentry crash/error telemetry when a non-empty DSN is provided.
pub fn init_sentry(dsn: &str, environment: &str, release: &str) -> Option<sentry::ClientInitGuard> {
    let trimmed = dsn.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut opts = sentry::ClientOptions::default();
    opts.release = Some(release.to_string().into());
    opts.environment = Some(environment.to_string().into());
    Some(sentry::init((trimmed, opts)))
}
