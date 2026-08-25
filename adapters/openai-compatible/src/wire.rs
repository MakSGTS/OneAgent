use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(crate) struct ModelsResponse {
    pub(crate) object: String,
    pub(crate) data: Vec<ModelEntry>,
}

#[derive(Deserialize)]
pub(crate) struct ModelEntry {
    pub(crate) id: String,
}

#[derive(Serialize)]
#[allow(dead_code)]
pub(crate) struct CompletionRequest<'a> {
    pub(crate) model: &'a str,
    pub(crate) prompt: &'a str,
    pub(crate) max_tokens: usize,
    pub(crate) stream: bool,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct CompletionResponse {
    pub(crate) object: String,
    pub(crate) model: String,
    pub(crate) choices: Vec<CompletionChoice>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct CompletionChoice {
    pub(crate) text: String,
    pub(crate) index: usize,
    pub(crate) finish_reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{CompletionRequest, CompletionResponse, ModelsResponse};

    #[test]
    fn completion_request_serializes_only_the_exact_accepted_fields() {
        let request = CompletionRequest {
            model: "model-a",
            prompt: "synthetic prompt",
            max_tokens: 17,
            stream: false,
        };
        let value = serde_json::to_value(request).expect("wire request must serialize");

        assert_eq!(
            value,
            json!({
                "model": "model-a",
                "prompt": "synthetic prompt",
                "max_tokens": 17,
                "stream": false
            })
        );
    }

    #[test]
    fn discovery_wire_requires_list_fields_and_ignores_unknown_additions() {
        let response: ModelsResponse = serde_json::from_value(json!({
            "object": "list",
            "data": [{"id": "model-a", "owned_by": "ignored"}],
            "future": true
        }))
        .expect("accepted discovery shape must decode");
        assert_eq!(response.object, "list");
        assert_eq!(response.data[0].id, "model-a");

        for invalid in [json!({"data": []}), json!({"object": "list"})] {
            assert!(serde_json::from_value::<ModelsResponse>(invalid).is_err());
        }
    }

    #[test]
    fn completion_wire_requires_terminal_fields_and_ignores_unknown_additions() {
        let response: CompletionResponse = serde_json::from_value(json!({
            "object": "text_completion",
            "model": "model-a",
            "choices": [{
                "text": "synthetic output",
                "index": 0,
                "finish_reason": "stop",
                "future": true
            }],
            "usage": {"ignored": true}
        }))
        .expect("accepted completion shape must decode");
        assert_eq!(response.object, "text_completion");
        assert_eq!(response.model, "model-a");
        assert_eq!(response.choices[0].text, "synthetic output");
        assert_eq!(response.choices[0].index, 0);
        assert_eq!(response.choices[0].finish_reason.as_deref(), Some("stop"));

        let missing: Value = json!({"object": "text_completion", "model": "model-a"});
        assert!(serde_json::from_value::<CompletionResponse>(missing).is_err());
    }
}
