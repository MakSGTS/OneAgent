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

#[derive(Serialize)]
pub(crate) struct GenerateRequest<'a> {
    pub(crate) model: &'a str,
    pub(crate) prompt: &'a str,
    pub(crate) stream: bool,
    pub(crate) raw: bool,
    pub(crate) think: bool,
    pub(crate) options: GenerationOptions,
}

#[derive(Serialize)]
pub(crate) struct GenerationOptions {
    pub(crate) num_predict: usize,
}

#[derive(Deserialize)]
pub(crate) struct GenerateResponse {
    pub(crate) model: String,
    pub(crate) response: String,
    pub(crate) done: bool,
    pub(crate) done_reason: String,
    #[serde(default)]
    pub(crate) thinking: OptionalString,
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{
        GenerateRequest, GenerateResponse, GenerationOptions, OptionalString, ShowRequest,
        ShowResponse, TagsResponse,
    };

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

    #[test]
    fn generate_request_serializes_only_the_exact_native_fields() {
        let body = serde_json::to_vec(&GenerateRequest {
            model: "model-a",
            prompt: "exact prompt",
            stream: false,
            raw: true,
            think: false,
            options: GenerationOptions { num_predict: 17 },
        })
        .expect("generate request must serialize");

        assert_eq!(
            body,
            br#"{"model":"model-a","prompt":"exact prompt","stream":false,"raw":true,"think":false,"options":{"num_predict":17}}"#
        );
    }

    #[test]
    fn generate_wire_requires_terminal_fields_and_rejects_null_thinking() {
        let response: GenerateResponse = serde_json::from_value(json!({
            "model": "model-a",
            "response": "output",
            "done": true,
            "done_reason": "stop",
            "future": true
        }))
        .expect("accepted generation shape must decode");
        assert_eq!(response.model, "model-a");
        assert!(matches!(response.thinking, OptionalString::Missing));

        for invalid in [
            json!({}),
            json!({"model": "model-a", "response": "output", "done": true}),
            json!({
                "model": "model-a", "response": "output", "done": "true", "done_reason": "stop"
            }),
            json!({
                "model": "model-a", "response": "output", "done": true,
                "done_reason": "stop", "thinking": null
            }),
        ] {
            assert!(serde_json::from_value::<GenerateResponse>(invalid).is_err());
        }
    }
}
