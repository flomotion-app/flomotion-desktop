use flomotion_desktop_lib::cli::uploads::PendingUpload;
use serde_json::json;

#[test]
fn an_ordinary_result_carries_no_upload() {
    let result = json!({ "listing_id": "L1", "status": "draft" });
    assert!(PendingUpload::find(&result).unwrap().is_none());
}

#[test]
fn the_envelope_is_parsed_whatever_the_tool_was() {
    let result = json!({ "upload": {
        "url": "https://signed/put",
        "headers": { "Content-Type": "image/webp" },
        "confirm_with": { "name": "some_future_tool", "input": { "id": "X" } }
    }});
    let upload = PendingUpload::find(&result).unwrap().unwrap();
    assert_eq!(upload.url, "https://signed/put");
    assert_eq!(upload.headers, vec![("Content-Type".to_string(), "image/webp".to_string())]);
    let (name, input) = upload.confirm_call().unwrap();
    assert_eq!(name, "some_future_tool");
    assert_eq!(input["id"], "X");
}

#[test]
fn headers_are_optional_but_must_be_strings_when_present() {
    let none = json!({ "upload": { "url": "u", "confirm_with": { "name": "t" } } });
    assert!(PendingUpload::find(&none).unwrap().unwrap().headers.is_empty());

    let bad = json!({ "upload": { "url": "u", "headers": { "Content-Length": 12 }, "confirm_with": { "name": "t" } } });
    let error = PendingUpload::find(&bad).unwrap_err();
    assert!(error.to_string().contains("Content-Length"), "{error}");
}

#[test]
fn an_envelope_without_a_url_or_confirm_is_refused() {
    let no_url = json!({ "upload": { "confirm_with": { "name": "t" } } });
    assert!(PendingUpload::find(&no_url).unwrap_err().to_string().contains("no url"));

    let no_confirm = json!({ "upload": { "url": "u" } });
    assert!(PendingUpload::find(&no_confirm).unwrap_err().to_string().contains("no confirm_with"));

    let nameless = json!({ "upload": { "url": "u", "confirm_with": { "input": {} } } });
    assert!(PendingUpload::find(&nameless).unwrap_err().to_string().contains("tool name"));
}

#[test]
fn a_confirm_without_input_is_an_empty_object() {
    let result = json!({ "upload": { "url": "u", "confirm_with": { "name": "t" } } });
    let (_, input) = PendingUpload::find(&result).unwrap().unwrap().confirm_call().unwrap();
    assert_eq!(input, json!({}));
}
