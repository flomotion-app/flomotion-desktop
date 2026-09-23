use crate::error::{DesktopError, Result};
use serde_json::Value;
use std::path::Path;

pub const UPLOAD_KEY: &str = "upload";

#[derive(Debug, Clone, PartialEq)]
pub struct PendingUpload {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub confirm_with: Value,
}

impl PendingUpload {
    pub fn find(result: &Value) -> Result<Option<Self>> {
        let Some(envelope) = result.get(UPLOAD_KEY) else {
            return Ok(None);
        };
        Self::parse(envelope).map(Some)
    }

    fn parse(envelope: &Value) -> Result<Self> {
        let url = Self::string(envelope, "url")?;
        let confirm_with = envelope
            .get("confirm_with")
            .cloned()
            .ok_or_else(|| Self::malformed("upload has no confirm_with"))?;
        if confirm_with.get("name").and_then(Value::as_str).is_none() {
            return Err(Self::malformed("upload confirm_with has no tool name"));
        }
        Ok(Self { url, headers: Self::headers(envelope)?, confirm_with })
    }

    fn headers(envelope: &Value) -> Result<Vec<(String, String)>> {
        let Some(headers) = envelope.get("headers") else {
            return Ok(vec![]);
        };
        let map = headers
            .as_object()
            .ok_or_else(|| Self::malformed("upload headers must be an object"))?;
        map.iter()
            .map(|(name, value)| {
                value
                    .as_str()
                    .map(|text| (name.clone(), text.to_owned()))
                    .ok_or_else(|| Self::malformed(&format!("upload header '{name}' is not a string")))
            })
            .collect()
    }

    fn string(envelope: &Value, key: &str) -> Result<String> {
        envelope
            .get(key)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| Self::malformed(&format!("upload has no {key}")))
    }

    pub fn confirm_call(&self) -> Result<(String, Value)> {
        let name = self.confirm_with["name"]
            .as_str()
            .ok_or_else(|| Self::malformed("upload confirm_with has no tool name"))?
            .to_owned();
        let input = self.confirm_with.get("input").cloned().unwrap_or_else(|| Value::Object(Default::default()));
        Ok((name, input))
    }

    fn malformed(detail: &str) -> DesktopError {
        DesktopError::Remote(format!("{detail}; the server sent an upload this version cannot perform"))
    }
}

pub trait UploadTransport: Send + Sync {
    fn put(&self, url: &str, headers: &[(String, String)], path: &Path) -> Result<()>;
}

pub struct HttpUploadTransport;

impl UploadTransport for HttpUploadTransport {
    fn put(&self, url: &str, headers: &[(String, String)], path: &Path) -> Result<()> {
        let file = std::fs::File::open(path)
            .map_err(|e| DesktopError::Remote(format!("cannot read {}: {e}", path.display())))?;
        let length = file.metadata()?.len();
        let mut request = ureq::put(url);
        for (name, value) in headers {
            request = request.set(name, value);
        }
        request = request.set("Content-Length", &length.to_string());
        request
            .send(std::io::BufReader::new(file))
            .map(|_| ())
            .map_err(Self::describe)
    }
}

impl HttpUploadTransport {
    fn describe(error: ureq::Error) -> DesktopError {
        match error {
            ureq::Error::Status(403, _) => DesktopError::Remote(
                "the upload was refused (403): the signed URL has expired or a signed header was altered; run the attach again".into(),
            ),
            ureq::Error::Status(400, response) => {
                let detail = response.into_string().unwrap_or_default();
                let hint = if detail.contains("MalformedSecurityHeader") {
                    ": a signed header was missing or changed"
                } else {
                    ""
                };
                DesktopError::Remote(format!("the upload was rejected (400){hint}"))
            }
            ureq::Error::Status(code, _) => DesktopError::Remote(format!("the upload failed with status {code}")),
            ureq::Error::Transport(transport) => DesktopError::Remote(format!("the upload could not be sent: {transport}")),
        }
    }
}
