//! Exact `/api/v1` HTTP projection for transport-neutral Graph Query operations.

use std::collections::BTreeMap;

use axum::extract::{RawQuery, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use serde::Serialize;

use crate::{
    GraphQueryConfiguration, GraphQueryDirection, GraphQueryEdgeKind, GraphQueryError,
    GraphQueryErrorKind, GraphQueryLimit, GraphQueryMaxDepth, GraphQueryNode, GraphQueryNodeResult,
    GraphQueryRelation, GraphQueryRelationResult, GraphQueryTraversalNode,
    GraphQueryTraversalResult,
};

use super::{HttpRouterState, get_only};

pub(super) fn routes() -> Router<HttpRouterState> {
    Router::new()
        .route("/api/v1/configurations", get_only(configurations))
        .route("/api/v1/graph/node", get_only(node))
        .route("/api/v1/graph/relations", get_only(relations))
        .route("/api/v1/graph/traverse", get_only(traverse))
}

async fn configurations(
    State(state): State<HttpRouterState>,
    RawQuery(raw_query): RawQuery,
) -> Response {
    let parameters = match Parameters::parse(raw_query.as_deref(), &["limit"], &[]) {
        Ok(parameters) => parameters,
        Err(error) => return error.into_response(),
    };
    let limit = match parse_limit(parameters.optional("limit")) {
        Ok(limit) => limit,
        Err(error) => return error.into_response(),
    };
    let query = match ready_query(&state) {
        Ok(query) => query,
        Err(error) => return error.into_response(),
    };

    match query.configurations(limit) {
        Ok(result) => Json(ConfigurationListResponse {
            configurations: result
                .configurations()
                .iter()
                .map(ConfigurationResponse::from)
                .collect(),
            truncated: result.truncated(),
        })
        .into_response(),
        Err(error) => ApiError::from(error).into_response(),
    }
}

async fn node(State(state): State<HttpRouterState>, RawQuery(raw_query): RawQuery) -> Response {
    let parameters = match Parameters::parse(
        raw_query.as_deref(),
        &["configuration_id", "node_id"],
        &["configuration_id", "node_id"],
    ) {
        Ok(parameters) => parameters,
        Err(error) => return error.into_response(),
    };
    let query = match ready_query(&state) {
        Ok(query) => query,
        Err(error) => return error.into_response(),
    };

    match query.node(
        parameters.required("configuration_id"),
        parameters.required("node_id"),
    ) {
        Ok(result) => Json(NodeLookupResponse::from(&result)).into_response(),
        Err(error) => ApiError::from(error).into_response(),
    }
}

async fn relations(
    State(state): State<HttpRouterState>,
    RawQuery(raw_query): RawQuery,
) -> Response {
    let parameters = match Parameters::parse(
        raw_query.as_deref(),
        &[
            "configuration_id",
            "node_id",
            "direction",
            "edge_kind",
            "limit",
        ],
        &["configuration_id", "node_id", "direction"],
    ) {
        Ok(parameters) => parameters,
        Err(error) => return error.into_response(),
    };
    let direction = match parse_direction(parameters.required("direction")) {
        Ok(direction) => direction,
        Err(error) => return error.into_response(),
    };
    let edge_kind = match parse_edge_kind(parameters.optional("edge_kind")) {
        Ok(edge_kind) => edge_kind,
        Err(error) => return error.into_response(),
    };
    let limit = match parse_limit(parameters.optional("limit")) {
        Ok(limit) => limit,
        Err(error) => return error.into_response(),
    };
    let query = match ready_query(&state) {
        Ok(query) => query,
        Err(error) => return error.into_response(),
    };

    match query.relations(
        parameters.required("configuration_id"),
        parameters.required("node_id"),
        direction,
        edge_kind,
        limit,
    ) {
        Ok(result) => Json(RelationListResponse::from(&result)).into_response(),
        Err(error) => ApiError::from(error).into_response(),
    }
}

async fn traverse(State(state): State<HttpRouterState>, RawQuery(raw_query): RawQuery) -> Response {
    let parameters = match Parameters::parse(
        raw_query.as_deref(),
        &[
            "configuration_id",
            "node_id",
            "direction",
            "max_depth",
            "edge_kind",
            "include_start",
            "limit",
        ],
        &["configuration_id", "node_id", "direction", "max_depth"],
    ) {
        Ok(parameters) => parameters,
        Err(error) => return error.into_response(),
    };
    let direction = match parse_direction(parameters.required("direction")) {
        Ok(direction) => direction,
        Err(error) => return error.into_response(),
    };
    let edge_kind = match parse_edge_kind(parameters.optional("edge_kind")) {
        Ok(edge_kind) => edge_kind,
        Err(error) => return error.into_response(),
    };
    let max_depth = match parse_max_depth(parameters.required("max_depth")) {
        Ok(max_depth) => max_depth,
        Err(error) => return error.into_response(),
    };
    let include_start = match parse_boolean(parameters.optional("include_start")) {
        Ok(include_start) => include_start,
        Err(error) => return error.into_response(),
    };
    let limit = match parse_limit(parameters.optional("limit")) {
        Ok(limit) => limit,
        Err(error) => return error.into_response(),
    };
    let query = match ready_query(&state) {
        Ok(query) => query,
        Err(error) => return error.into_response(),
    };

    match query.traverse(
        parameters.required("configuration_id"),
        parameters.required("node_id"),
        direction,
        edge_kind,
        max_depth,
        include_start,
        limit,
    ) {
        Ok(result) => Json(TraversalResponse::from(&result)).into_response(),
        Err(error) => ApiError::from(error).into_response(),
    }
}

fn ready_query(state: &HttpRouterState) -> Result<&crate::GraphQueryService, ApiError> {
    if !state.app.health().snapshot().is_ready() {
        return Err(ApiError::RuntimeNotReady);
    }
    Ok(state
        .graph_query
        .as_ref()
        .expect("Graph Query routes require a query-enabled HTTP service"))
}

#[derive(Debug)]
struct Parameters {
    values: BTreeMap<String, String>,
}

impl Parameters {
    fn parse(raw: Option<&str>, allowed: &[&str], required: &[&str]) -> Result<Self, ApiError> {
        let mut values = BTreeMap::new();
        if let Some(raw) = raw.filter(|raw| !raw.is_empty()) {
            for pair in raw.split('&') {
                let (name, value) = pair.split_once('=').ok_or(ApiError::InvalidQuery)?;
                if name.is_empty() {
                    return Err(ApiError::InvalidQuery);
                }
                let name = decode_component(name)?;
                let value = decode_component(value)?;
                if !allowed.contains(&name.as_str()) || values.insert(name, value).is_some() {
                    return Err(ApiError::InvalidQuery);
                }
            }
        }

        if required
            .iter()
            .any(|name| values.get(*name).is_none_or(String::is_empty))
        {
            return Err(ApiError::InvalidQuery);
        }
        Ok(Self { values })
    }

    fn required(&self, name: &str) -> &str {
        self.values
            .get(name)
            .expect("required parameters are validated")
    }

    fn optional(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(String::as_str)
    }
}

fn decode_component(value: &str) -> Result<String, ApiError> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' => {
                let high = bytes.get(index + 1).and_then(|value| hex(*value));
                let low = bytes.get(index + 2).and_then(|value| hex(*value));
                let (Some(high), Some(low)) = (high, low) else {
                    return Err(ApiError::InvalidQuery);
                };
                decoded.push((high << 4) | low);
                index += 3;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(decoded).map_err(|_| ApiError::InvalidQuery)
}

const fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn parse_direction(value: &str) -> Result<GraphQueryDirection, ApiError> {
    GraphQueryDirection::from_name(value).ok_or(ApiError::UnsupportedDirection)
}

fn parse_edge_kind(value: Option<&str>) -> Result<Option<GraphQueryEdgeKind>, ApiError> {
    value
        .map(|value| GraphQueryEdgeKind::from_name(value).ok_or(ApiError::UnsupportedEdgeKind))
        .transpose()
}

fn parse_limit(value: Option<&str>) -> Result<GraphQueryLimit, ApiError> {
    let Some(value) = value else {
        return Ok(GraphQueryLimit::default());
    };
    let parsed = parse_unsigned(value).ok_or(ApiError::LimitOutOfRange)?;
    GraphQueryLimit::new(parsed).map_err(|_| ApiError::LimitOutOfRange)
}

fn parse_max_depth(value: &str) -> Result<GraphQueryMaxDepth, ApiError> {
    let parsed = parse_unsigned(value).ok_or(ApiError::MaxDepthOutOfRange)?;
    GraphQueryMaxDepth::new(parsed).map_err(|_| ApiError::MaxDepthOutOfRange)
}

fn parse_unsigned(value: &str) -> Option<usize> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

fn parse_boolean(value: Option<&str>) -> Result<bool, ApiError> {
    match value {
        None | Some("false") => Ok(false),
        Some("true") => Ok(true),
        Some(_) => Err(ApiError::InvalidBoolean),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApiError {
    RuntimeNotReady,
    WorkspaceUnavailable,
    ConfigurationNotFound,
    NodeNotFound,
    InvalidIdentifier,
    InvalidQuery,
    UnsupportedDirection,
    UnsupportedEdgeKind,
    LimitOutOfRange,
    MaxDepthOutOfRange,
    InvalidBoolean,
}

impl From<GraphQueryError> for ApiError {
    fn from(error: GraphQueryError) -> Self {
        match error.kind() {
            GraphQueryErrorKind::WorkspaceUnavailable => Self::WorkspaceUnavailable,
            GraphQueryErrorKind::InvalidIdentifier => Self::InvalidIdentifier,
            GraphQueryErrorKind::ConfigurationNotFound => Self::ConfigurationNotFound,
            GraphQueryErrorKind::NodeNotFound => Self::NodeNotFound,
            GraphQueryErrorKind::LimitOutOfRange => Self::LimitOutOfRange,
            GraphQueryErrorKind::MaxDepthOutOfRange => Self::MaxDepthOutOfRange,
        }
    }
}

impl ApiError {
    const fn contract(self) -> (StatusCode, &'static str, &'static str) {
        match self {
            Self::RuntimeNotReady => (
                StatusCode::SERVICE_UNAVAILABLE,
                "runtime_not_ready",
                "runtime is not ready",
            ),
            Self::WorkspaceUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "workspace_unavailable",
                "workspace snapshot is unavailable",
            ),
            Self::ConfigurationNotFound => (
                StatusCode::NOT_FOUND,
                "configuration_not_found",
                "configuration was not found",
            ),
            Self::NodeNotFound => (
                StatusCode::NOT_FOUND,
                "node_not_found",
                "node was not found",
            ),
            Self::InvalidIdentifier => (
                StatusCode::BAD_REQUEST,
                "invalid_identifier",
                "identifier must not be empty",
            ),
            Self::InvalidQuery => (
                StatusCode::BAD_REQUEST,
                "invalid_query",
                "query parameters are invalid",
            ),
            Self::UnsupportedDirection => (
                StatusCode::BAD_REQUEST,
                "unsupported_direction",
                "direction is unsupported",
            ),
            Self::UnsupportedEdgeKind => (
                StatusCode::BAD_REQUEST,
                "unsupported_edge_kind",
                "edge kind is unsupported",
            ),
            Self::LimitOutOfRange => (
                StatusCode::BAD_REQUEST,
                "limit_out_of_range",
                "limit must be between 1 and 100",
            ),
            Self::MaxDepthOutOfRange => (
                StatusCode::BAD_REQUEST,
                "max_depth_out_of_range",
                "max_depth must be between 0 and 4",
            ),
            Self::InvalidBoolean => (
                StatusCode::BAD_REQUEST,
                "invalid_boolean",
                "include_start must be true or false",
            ),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = self.contract();
        (
            status,
            Json(ErrorEnvelope {
                error: ErrorResponse { code, message },
            }),
        )
            .into_response()
    }
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    error: ErrorResponse,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: &'static str,
    message: &'static str,
}

#[derive(Debug, Serialize)]
struct ConfigurationListResponse {
    configurations: Vec<ConfigurationResponse>,
    truncated: bool,
}

#[derive(Debug, Serialize)]
struct ConfigurationResponse {
    id: String,
    name: String,
    format: &'static str,
    node_count: usize,
    edge_count: usize,
}

impl From<&GraphQueryConfiguration> for ConfigurationResponse {
    fn from(value: &GraphQueryConfiguration) -> Self {
        Self {
            id: value.id().to_owned(),
            name: value.name().to_owned(),
            format: value.format().as_str(),
            node_count: value.node_count(),
            edge_count: value.edge_count(),
        }
    }
}

#[derive(Debug, Serialize)]
struct NodeLookupResponse {
    configuration_id: String,
    node: NodeResponse,
}

impl From<&GraphQueryNodeResult> for NodeLookupResponse {
    fn from(value: &GraphQueryNodeResult) -> Self {
        Self {
            configuration_id: value.configuration_id().to_owned(),
            node: NodeResponse::from(value.node()),
        }
    }
}

#[derive(Debug, Serialize)]
struct NodeResponse {
    id: String,
    name: String,
    kind: &'static str,
    metadata_kind: Option<&'static str>,
}

impl From<&GraphQueryNode> for NodeResponse {
    fn from(value: &GraphQueryNode) -> Self {
        Self {
            id: value.id().to_owned(),
            name: value.name().to_owned(),
            kind: value.kind().as_str(),
            metadata_kind: value
                .kind()
                .metadata_kind()
                .map(crate::GraphQueryMetadataKind::as_str),
        }
    }
}

#[derive(Debug, Serialize)]
struct RelationListResponse {
    configuration_id: String,
    node_id: String,
    direction: &'static str,
    edge_kind: Option<&'static str>,
    relations: Vec<RelationResponse>,
    truncated: bool,
}

impl From<&GraphQueryRelationResult> for RelationListResponse {
    fn from(value: &GraphQueryRelationResult) -> Self {
        Self {
            configuration_id: value.configuration_id().to_owned(),
            node_id: value.node_id().to_owned(),
            direction: value.direction().as_str(),
            edge_kind: value.edge_kind().map(GraphQueryEdgeKind::as_str),
            relations: value
                .relations()
                .iter()
                .map(RelationResponse::from)
                .collect(),
            truncated: value.truncated(),
        }
    }
}

#[derive(Debug, Serialize)]
struct RelationResponse {
    edge_id: String,
    edge_kind: &'static str,
    source_node_id: String,
    target_node_id: String,
    related_node: NodeResponse,
}

impl From<&GraphQueryRelation> for RelationResponse {
    fn from(value: &GraphQueryRelation) -> Self {
        Self {
            edge_id: value.edge_id().to_owned(),
            edge_kind: value.edge_kind().as_str(),
            source_node_id: value.source_node_id().to_owned(),
            target_node_id: value.target_node_id().to_owned(),
            related_node: NodeResponse::from(value.related_node()),
        }
    }
}

#[derive(Debug, Serialize)]
struct TraversalResponse {
    configuration_id: String,
    start_node_id: String,
    direction: &'static str,
    edge_kind: Option<&'static str>,
    max_depth: usize,
    include_start: bool,
    nodes: Vec<TraversalNodeResponse>,
    truncated: bool,
}

impl From<&GraphQueryTraversalResult> for TraversalResponse {
    fn from(value: &GraphQueryTraversalResult) -> Self {
        Self {
            configuration_id: value.configuration_id().to_owned(),
            start_node_id: value.start_node_id().to_owned(),
            direction: value.direction().as_str(),
            edge_kind: value.edge_kind().map(GraphQueryEdgeKind::as_str),
            max_depth: value.max_depth(),
            include_start: value.include_start(),
            nodes: value
                .nodes()
                .iter()
                .map(TraversalNodeResponse::from)
                .collect(),
            truncated: value.truncated(),
        }
    }
}

#[derive(Debug, Serialize)]
struct TraversalNodeResponse {
    node: NodeResponse,
    depth: usize,
    via_edge_id: Option<String>,
}

impl From<&GraphQueryTraversalNode> for TraversalNodeResponse {
    fn from(value: &GraphQueryTraversalNode) -> Self {
        Self {
            node: NodeResponse::from(value.node()),
            depth: value.depth(),
            via_edge_id: value.via_edge_id().map(ToOwned::to_owned),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ApiError, Parameters, decode_component, parse_boolean, parse_direction, parse_edge_kind,
        parse_limit, parse_max_depth,
    };

    #[test]
    fn strict_query_structure_decodes_once_and_rejects_ambiguity() {
        let parameters = Parameters::parse(
            Some("configuration%5fid=hello+world&node_id=node%2Fone"),
            &["configuration_id", "node_id"],
            &["configuration_id", "node_id"],
        )
        .expect("valid encoded parameters must parse");
        assert_eq!(parameters.required("configuration_id"), "hello world");
        assert_eq!(parameters.required("node_id"), "node/one");
        assert_eq!(decode_component("%252F"), Ok("%2F".to_owned()));

        for raw in [
            "configuration_id=a&configuration_id=b&node_id=n",
            "configuration_id=a&node_id=n&unknown=x",
            "configuration_id=a",
            "configuration_id=&node_id=n",
            "configuration_id=a&node_id=%",
            "configuration_id=a&node_id=%FF",
            "configuration_id=a&node_id=n&",
            "configuration_id=a&node_id",
        ] {
            assert_eq!(
                Parameters::parse(
                    Some(raw),
                    &["configuration_id", "node_id"],
                    &["configuration_id", "node_id"],
                )
                .expect_err("structurally invalid query must fail"),
                ApiError::InvalidQuery
            );
        }
    }

    #[test]
    fn closed_values_defaults_and_bounds_are_exact() {
        assert_eq!(
            parse_direction("outgoing")
                .expect("direction must parse")
                .as_str(),
            "outgoing"
        );
        assert_eq!(
            parse_direction("Outgoing"),
            Err(ApiError::UnsupportedDirection)
        );
        assert_eq!(
            parse_edge_kind(Some("depends_on"))
                .expect("edge kind must parse")
                .expect("edge kind must be present")
                .as_str(),
            "depends_on"
        );
        assert_eq!(parse_edge_kind(None), Ok(None));
        assert_eq!(
            parse_edge_kind(Some("dependency")),
            Err(ApiError::UnsupportedEdgeKind)
        );

        assert_eq!(
            parse_limit(None).expect("default limit must parse").get(),
            50
        );
        assert_eq!(
            parse_limit(Some("001"))
                .expect("leading zeroes must parse")
                .get(),
            1
        );
        assert_eq!(
            parse_limit(Some("100")).expect("maximum must parse").get(),
            100
        );
        for value in ["", "0", "101", "+1", " 1", "1.0", "184467440737095516160"] {
            assert_eq!(parse_limit(Some(value)), Err(ApiError::LimitOutOfRange));
        }

        assert_eq!(
            parse_max_depth("0").expect("zero depth must parse").get(),
            0
        );
        assert_eq!(
            parse_max_depth("4")
                .expect("maximum depth must parse")
                .get(),
            4
        );
        for value in ["", "5", "-1", " 1", "184467440737095516160"] {
            assert_eq!(parse_max_depth(value), Err(ApiError::MaxDepthOutOfRange));
        }

        assert_eq!(parse_boolean(None), Ok(false));
        assert_eq!(parse_boolean(Some("false")), Ok(false));
        assert_eq!(parse_boolean(Some("true")), Ok(true));
        assert_eq!(parse_boolean(Some("TRUE")), Err(ApiError::InvalidBoolean));
    }

    #[test]
    fn every_error_has_the_exact_closed_wire_contract() {
        let rows = [
            (
                ApiError::RuntimeNotReady,
                503,
                "runtime_not_ready",
                "runtime is not ready",
            ),
            (
                ApiError::WorkspaceUnavailable,
                503,
                "workspace_unavailable",
                "workspace snapshot is unavailable",
            ),
            (
                ApiError::ConfigurationNotFound,
                404,
                "configuration_not_found",
                "configuration was not found",
            ),
            (
                ApiError::NodeNotFound,
                404,
                "node_not_found",
                "node was not found",
            ),
            (
                ApiError::InvalidIdentifier,
                400,
                "invalid_identifier",
                "identifier must not be empty",
            ),
            (
                ApiError::InvalidQuery,
                400,
                "invalid_query",
                "query parameters are invalid",
            ),
            (
                ApiError::UnsupportedDirection,
                400,
                "unsupported_direction",
                "direction is unsupported",
            ),
            (
                ApiError::UnsupportedEdgeKind,
                400,
                "unsupported_edge_kind",
                "edge kind is unsupported",
            ),
            (
                ApiError::LimitOutOfRange,
                400,
                "limit_out_of_range",
                "limit must be between 1 and 100",
            ),
            (
                ApiError::MaxDepthOutOfRange,
                400,
                "max_depth_out_of_range",
                "max_depth must be between 0 and 4",
            ),
            (
                ApiError::InvalidBoolean,
                400,
                "invalid_boolean",
                "include_start must be true or false",
            ),
        ];
        for (error, status, code, message) in rows {
            let contract = error.contract();
            assert_eq!(contract.0.as_u16(), status);
            assert_eq!(contract.1, code);
            assert_eq!(contract.2, message);
        }
    }
}
