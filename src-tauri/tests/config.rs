use flomotion_desktop_lib::config::AppConfig;

const FILE: &str = r#"{"web_url": "https://feature.flomotion.app/"}"#;

#[test]
fn environment_beats_file_beats_default() {
    assert_eq!(AppConfig::resolve(None, None).start_url(), "https://flomotion.app/projects");
    assert_eq!(AppConfig::resolve(None, Some(FILE)).start_url(), "https://feature.flomotion.app/projects");
    assert!(AppConfig::resolve(Some("local".into()), Some(FILE)).uses_local_page());
}

#[test]
fn empty_or_malformed_values_fall_back() {
    let config = AppConfig::resolve(Some(String::new()), Some("not json"));
    assert_eq!(config.web_url, "https://flomotion.app");
}
