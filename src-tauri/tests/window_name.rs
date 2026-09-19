use flomotion_desktop_lib::window_name::{WindowName, DEFAULT_WINDOW};

#[test]
fn the_default_keeps_todays_socket_file() {
    let window = WindowName::default();
    assert_eq!(window.as_str(), DEFAULT_WINDOW);
    assert_eq!(window.socket_file(), "flomotion.sock");
    assert!(window.is_default());
}

#[test]
fn a_named_window_gets_its_own_socket_file() {
    let window = WindowName::parse("data-entry").unwrap();
    assert_eq!(window.socket_file(), "flomotion-data-entry.sock");
    assert!(!window.is_default());
}

#[test]
fn naming_a_window_flomotion_is_the_default() {
    let window = WindowName::parse("flomotion").unwrap();
    assert_eq!(window.socket_file(), "flomotion.sock");
    assert!(window.is_default());
}

#[test]
fn the_flag_wins_over_the_environment_which_wins_over_the_default() {
    assert_eq!(WindowName::resolve(Some("from-flag"), Some("from-env".into())).unwrap().as_str(), "from-flag");
    assert_eq!(WindowName::resolve(None, Some("from-env".into())).unwrap().as_str(), "from-env");
    assert_eq!(WindowName::resolve(None, None).unwrap().as_str(), DEFAULT_WINDOW);
    assert_eq!(WindowName::resolve(None, Some(String::new())).unwrap().as_str(), DEFAULT_WINDOW);
}

#[test]
fn unsafe_names_are_rejected() {
    for bad in ["../escape", "with space", "slash/name", "dot.name", "", &"x".repeat(33)] {
        assert!(WindowName::parse(bad).is_err(), "{bad:?} should be rejected");
    }
}

#[test]
fn allowed_characters_are_letters_digits_underscore_and_dash() {
    for good in ["a", "Edge_2", "data-entry", &"x".repeat(32)] {
        assert!(WindowName::parse(good).is_ok(), "{good:?} should be allowed");
    }
}

#[test]
fn the_launcher_passes_the_window_name_to_the_spawned_app() {
    use flomotion_desktop_lib::cli::launcher::ProcessLauncher;
    assert_eq!(ProcessLauncher::args(&WindowName::default()), vec!["--window", "flomotion"]);
    assert_eq!(
        ProcessLauncher::args(&WindowName::parse("data-entry").unwrap()),
        vec!["--window", "data-entry"]
    );
}
