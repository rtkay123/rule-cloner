pub fn build_name(
    scope: Option<&str>,
    registry: &str,
    rule: &str,
    prefix: &str,
    version: &str,
) -> String {
    let scope = scope
        .as_ref()
        .and_then(|f| format!("{f}/").into())
        .unwrap_or_default();

    format!("{registry}:{scope}{}{rule}{}", prefix, version)
}
