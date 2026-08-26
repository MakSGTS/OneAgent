use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct ModelsResponse {
    pub(crate) models: Vec<ModelEntry>,
}

#[derive(Deserialize)]
pub(crate) struct ModelEntry {
    #[serde(rename = "type")]
    pub(crate) model_type: String,
    pub(crate) key: String,
    pub(crate) loaded_instances: Vec<LoadedInstance>,
}

#[derive(Deserialize)]
pub(crate) struct LoadedInstance {
    pub(crate) id: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::ModelsResponse;

    #[test]
    fn discovery_wire_requires_native_fields_and_ignores_unknown_additions() {
        let response: ModelsResponse = serde_json::from_value(json!({
            "models": [{
                "type": "llm",
                "key": "downloaded-model",
                "loaded_instances": [{"id": "loaded-model", "future": true}],
                "publisher": "ignored"
            }],
            "future": true
        }))
        .expect("accepted native discovery shape must decode");

        assert_eq!(response.models[0].model_type, "llm");
        assert_eq!(response.models[0].key, "downloaded-model");
        assert_eq!(response.models[0].loaded_instances[0].id, "loaded-model");

        for invalid in [
            json!({}),
            json!({"models": {}}),
            json!({"models": [{}]}),
            json!({"models": [{"type": "llm", "loaded_instances": []}]}),
            json!({"models": [{"type": "llm", "key": "model"}]}),
            json!({
                "models": [{
                    "type": "llm",
                    "key": "model",
                    "loaded_instances": [{}]
                }]
            }),
        ] {
            assert!(serde_json::from_value::<ModelsResponse>(invalid).is_err());
        }
    }
}
