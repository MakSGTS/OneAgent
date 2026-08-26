use std::fmt::Formatter;

use serde::{Deserialize, Deserializer, Serialize, de::Visitor};

#[derive(Deserialize)]
pub(crate) struct TagsResponse {
    pub(crate) models: Vec<TagEntry>,
}

#[derive(Deserialize)]
pub(crate) struct TagEntry {
    pub(crate) name: String,
    pub(crate) model: String,
    #[serde(default)]
    pub(crate) remote_model: OptionalString,
    #[serde(default)]
    pub(crate) remote_host: OptionalString,
}

#[derive(Default)]
pub(crate) enum OptionalString {
    #[default]
    Missing,
    Present(String),
}

impl<'de> Deserialize<'de> for OptionalString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringVisitor;

        impl Visitor<'_> for StringVisitor {
            type Value = OptionalString;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(OptionalString::Present(value.to_owned()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(OptionalString::Present(value))
            }
        }

        deserializer.deserialize_string(StringVisitor)
    }
}

#[derive(Serialize)]
pub(crate) struct ShowRequest<'a> {
    pub(crate) model: &'a str,
    pub(crate) verbose: bool,
}

#[derive(Deserialize)]
pub(crate) struct ShowResponse {
    pub(crate) capabilities: Vec<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{OptionalString, ShowRequest, ShowResponse, TagsResponse};

    #[test]
    fn show_request_serializes_only_the_exact_accepted_fields() {
        let body = serde_json::to_vec(&ShowRequest {
            model: "model-a",
            verbose: false,
        })
        .expect("show request must serialize");

        assert_eq!(body, br#"{"model":"model-a","verbose":false}"#);
    }

    #[test]
    fn tags_wire_requires_identity_and_distinguishes_missing_remote_markers() {
        let response: TagsResponse = serde_json::from_value(json!({
            "models": [{
                "name": "local-model",
                "model": "local-model",
                "capabilities": ["completion"],
                "future": true
            }, {
                "name": "remote-model",
                "model": "remote-model",
                "remote_model": "opaque-remote-model",
                "remote_host": "opaque-remote-host"
            }],
            "future": true
        }))
        .expect("accepted tags shape must decode");

        assert!(matches!(
            response.models[0].remote_model,
            OptionalString::Missing
        ));
        assert!(matches!(
            response.models[1].remote_host,
            OptionalString::Present(_)
        ));

        for invalid in [
            json!({}),
            json!({"models": {}}),
            json!({"models": [{}]}),
            json!({"models": [{"name": "model"}]}),
            json!({"models": [{"name": "model", "model": "model", "remote_model": null}]}),
            json!({"models": [{"name": "model", "model": "model", "remote_host": 1}]}),
        ] {
            assert!(serde_json::from_value::<TagsResponse>(invalid).is_err());
        }
    }

    #[test]
    fn show_wire_requires_a_string_capability_array_and_ignores_additions() {
        let response: ShowResponse = serde_json::from_value(json!({
            "capabilities": ["completion", "tools"],
            "future": true
        }))
        .expect("accepted show shape must decode");
        assert_eq!(response.capabilities, ["completion", "tools"]);

        for invalid in [
            json!({}),
            json!({"capabilities": null}),
            json!({"capabilities": {}}),
            json!({"capabilities": ["completion", 1]}),
        ] {
            assert!(serde_json::from_value::<ShowResponse>(invalid).is_err());
        }

        let value: Value = serde_json::to_value(ShowRequest {
            model: "model-a",
            verbose: false,
        })
        .expect("request must serialize");
        assert_eq!(value, json!({"model": "model-a", "verbose": false}));
    }
}
