use flomotion_desktop_lib::ipc::name::SocketName;
use flomotion_desktop_lib::window_name::WindowName;

#[test]
fn the_default_and_a_named_window_resolve_to_different_sockets() {
    let default = SocketName::resolve(&WindowName::default()).unwrap();
    let named = SocketName::resolve(&WindowName::parse("edge").unwrap()).unwrap();
    assert_ne!(format!("{default:?}"), format!("{named:?}"));
}

#[test]
fn resolution_is_stable_for_the_same_name() {
    let first = SocketName::resolve(&WindowName::parse("edge").unwrap()).unwrap();
    let second = SocketName::resolve(&WindowName::parse("edge").unwrap()).unwrap();
    assert_eq!(format!("{first:?}"), format!("{second:?}"));
}

#[cfg(not(windows))]
#[test]
fn unix_paths_carry_the_window_name_and_stay_under_one_directory() {
    let default = SocketName::socket_path(&WindowName::default()).unwrap();
    let named = SocketName::socket_path(&WindowName::parse("edge").unwrap()).unwrap();
    assert_eq!(default.file_name().unwrap(), "flomotion.sock");
    assert_eq!(named.file_name().unwrap(), "flomotion-edge.sock");
    assert_eq!(default.parent(), named.parent());
}
