use flomotion_desktop_lib::ipc::protocol::{Request, Response};
use serde_json::json;

#[test]
fn request_round_trips_through_json() {
    let request = Request::new("act", json!({"name": "list_projects"}));
    let text = serde_json::to_string(&request).unwrap();
    assert_eq!(serde_json::from_str::<Request>(&text).unwrap(), request);
}

#[test]
fn failure_response_carries_error() {
    let text = serde_json::to_string(&Response::failure("boom")).unwrap();
    assert_eq!(text, r#"{"ok":false,"result":null,"error":"boom"}"#);
}

#[test]
fn missing_payload_defaults_to_null() {
    let request: Request = serde_json::from_str(r#"{"command": "status"}"#).unwrap();
    assert_eq!(request, Request::status());
}
