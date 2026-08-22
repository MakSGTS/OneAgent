//! Bounded HTTP/1.1 transport for the supported Runtime client.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::{ClientRequest, Command, ExecutionOutcome, HealthProbe, RequestExecutor};

const SOCKET_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_HEAD_BYTES: usize = 64 * 1024;
const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;
const READ_BUFFER_BYTES: usize = 8 * 1024;

/// Dependency-free blocking executor for accepted Runtime HTTP requests.
#[derive(Debug, Default, Clone, Copy)]
pub struct HttpExecutor;

impl RequestExecutor for HttpExecutor {
    fn execute(&mut self, request: &ClientRequest) -> ExecutionOutcome {
        execute(request).unwrap_or_else(|failure| match failure {
            ClientFailure::Transport => ExecutionOutcome::TransportFailure,
            ClientFailure::Protocol => ExecutionOutcome::ProtocolFailure,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClientFailure {
    Transport,
    Protocol,
}

fn execute(request: &ClientRequest) -> Result<ExecutionOutcome, ClientFailure> {
    let target = request_target(request.command());
    let request_bytes = request_bytes(request, &target);
    let mut stream = TcpStream::connect_timeout(&request.address(), SOCKET_TIMEOUT)
        .map_err(|_| ClientFailure::Transport)?;
    stream
        .set_read_timeout(Some(SOCKET_TIMEOUT))
        .and_then(|()| stream.set_write_timeout(Some(SOCKET_TIMEOUT)))
        .map_err(|_| ClientFailure::Transport)?;
    stream
        .write_all(&request_bytes)
        .and_then(|()| stream.flush())
        .map_err(|_| ClientFailure::Transport)?;
    let response = read_response(&mut stream)?;
    parse_response(&response)
}

fn request_target(command: &Command) -> String {
    match command {
        Command::Health(HealthProbe::Live) => "/health/live".to_owned(),
        Command::Health(HealthProbe::Ready) => "/health/ready".to_owned(),
        Command::Configurations { limit } => {
            let mut query = Query::new("/api/v1/configurations");
            query.optional_usize("limit", *limit);
            query.finish()
        }
        Command::Node {
            configuration_id,
            node_id,
        } => {
            let mut query = Query::new("/api/v1/graph/node");
            query.string("configuration_id", configuration_id);
            query.string("node_id", node_id);
            query.finish()
        }
        Command::Relations {
            configuration_id,
            node_id,
            direction,
            edge_kind,
            limit,
        } => {
            let mut query = Query::new("/api/v1/graph/relations");
            query.string("configuration_id", configuration_id);
            query.string("node_id", node_id);
            query.string("direction", direction.as_str());
            query.optional_string("edge_kind", edge_kind.as_deref());
            query.optional_usize("limit", *limit);
            query.finish()
        }
        Command::Traverse {
            configuration_id,
            node_id,
            direction,
            max_depth,
            edge_kind,
            include_start,
            limit,
        } => {
            let mut query = Query::new("/api/v1/graph/traverse");
            query.string("configuration_id", configuration_id);
            query.string("node_id", node_id);
            query.string("direction", direction.as_str());
            query.usize("max_depth", *max_depth);
            query.optional_string("edge_kind", edge_kind.as_deref());
            if *include_start {
                query.string("include_start", "true");
            }
            query.optional_usize("limit", *limit);
            query.finish()
        }
    }
}

fn request_bytes(request: &ClientRequest, target: &str) -> Vec<u8> {
    format!(
        "GET {target} HTTP/1.1\r\nHost: {}\r\nAccept: application/json\r\nConnection: close\r\n\r\n",
        request.address()
    )
    .into_bytes()
}

fn read_response(stream: &mut TcpStream) -> Result<Vec<u8>, ClientFailure> {
    let mut response = Vec::new();
    let mut buffer = [0_u8; READ_BUFFER_BYTES];
    let mut head_end = None;
    loop {
        let read = stream
            .read(&mut buffer)
            .map_err(|_| ClientFailure::Transport)?;
        if read == 0 {
            break;
        }
        response.extend_from_slice(&buffer[..read]);
        if head_end.is_none() {
            head_end = find_head_end(&response);
            if head_end.is_none() && response.len() > MAX_HEAD_BYTES {
                return Err(ClientFailure::Protocol);
            }
        }
        if let Some(end) = head_end
            && (end > MAX_HEAD_BYTES || response.len().saturating_sub(end) > MAX_BODY_BYTES)
        {
            return Err(ClientFailure::Protocol);
        }
    }
    Ok(response)
}

fn parse_response(response: &[u8]) -> Result<ExecutionOutcome, ClientFailure> {
    let head_end = find_head_end(response).ok_or(ClientFailure::Protocol)?;
    if head_end > MAX_HEAD_BYTES {
        return Err(ClientFailure::Protocol);
    }
    let body = &response[head_end..];
    if body.is_empty() || body.len() > MAX_BODY_BYTES || std::str::from_utf8(body).is_err() {
        return Err(ClientFailure::Protocol);
    }
    let head =
        std::str::from_utf8(&response[..head_end - 4]).map_err(|_| ClientFailure::Protocol)?;
    if !head.is_ascii() {
        return Err(ClientFailure::Protocol);
    }
    let mut lines = head.split("\r\n");
    let status = parse_status_line(lines.next().ok_or(ClientFailure::Protocol)?)?;
    let mut content_type = None;
    let mut content_length = None;
    for line in lines {
        let (name, value) = line.split_once(':').ok_or(ClientFailure::Protocol)?;
        if name.is_empty()
            || !name.bytes().all(is_header_name_byte)
            || value.bytes().any(is_invalid_header_value_byte)
        {
            return Err(ClientFailure::Protocol);
        }
        let value = value.trim_matches([' ', '\t']);
        if name.eq_ignore_ascii_case("transfer-encoding") {
            return Err(ClientFailure::Protocol);
        }
        if name.eq_ignore_ascii_case("content-type") {
            if content_type.replace(value).is_some() {
                return Err(ClientFailure::Protocol);
            }
        } else if name.eq_ignore_ascii_case("content-length") {
            if content_length.is_some()
                || value.is_empty()
                || !value.bytes().all(|byte| byte.is_ascii_digit())
            {
                return Err(ClientFailure::Protocol);
            }
            content_length = Some(
                value
                    .parse::<usize>()
                    .map_err(|_| ClientFailure::Protocol)?,
            );
        }
    }
    if !content_type.is_some_and(|value| value.eq_ignore_ascii_case("application/json")) {
        return Err(ClientFailure::Protocol);
    }
    if content_length.is_some_and(|length| length != body.len()) {
        return Err(ClientFailure::Protocol);
    }
    if status == 200 {
        Ok(ExecutionOutcome::Success(body.to_vec()))
    } else {
        Ok(ExecutionOutcome::ServerFailure(body.to_vec()))
    }
}

fn find_head_end(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
}

fn parse_status_line(line: &str) -> Result<u16, ClientFailure> {
    let mut parts = line.split_ascii_whitespace();
    let version = parts.next().ok_or(ClientFailure::Protocol)?;
    if !matches!(version, "HTTP/1.0" | "HTTP/1.1") {
        return Err(ClientFailure::Protocol);
    }
    let code = parts.next().ok_or(ClientFailure::Protocol)?;
    if code.len() != 3 || !code.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ClientFailure::Protocol);
    }
    let code = code.parse::<u16>().map_err(|_| ClientFailure::Protocol)?;
    if !(100..=599).contains(&code) {
        return Err(ClientFailure::Protocol);
    }
    Ok(code)
}

fn is_header_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

fn is_invalid_header_value_byte(byte: u8) -> bool {
    byte.is_ascii_control() && byte != b'\t'
}

struct Query {
    target: String,
    has_parameter: bool,
}

impl Query {
    fn new(path: &str) -> Self {
        Self {
            target: path.to_owned(),
            has_parameter: false,
        }
    }

    fn string(&mut self, name: &str, value: &str) {
        self.separator();
        self.target.push_str(name);
        self.target.push('=');
        percent_encode_into(value.as_bytes(), &mut self.target);
    }

    fn usize(&mut self, name: &str, value: usize) {
        self.separator();
        self.target.push_str(name);
        self.target.push('=');
        self.target.push_str(&value.to_string());
    }

    fn optional_string(&mut self, name: &str, value: Option<&str>) {
        if let Some(value) = value {
            self.string(name, value);
        }
    }

    fn optional_usize(&mut self, name: &str, value: Option<usize>) {
        if let Some(value) = value {
            self.usize(name, value);
        }
    }

    fn separator(&mut self) {
        self.target.push(if self.has_parameter { '&' } else { '?' });
        self.has_parameter = true;
    }

    fn finish(self) -> String {
        self.target
    }
}

fn percent_encode_into(bytes: &[u8], output: &mut String) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for &byte in bytes {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            output.push(char::from(byte));
        } else {
            output.push('%');
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener};
    use std::thread;

    use crate::Direction;

    use super::*;

    #[test]
    fn request_targets_cover_every_command_in_exact_parameter_order() {
        let cases = [
            (Command::Health(HealthProbe::Live), "/health/live"),
            (Command::Health(HealthProbe::Ready), "/health/ready"),
            (
                Command::Configurations { limit: None },
                "/api/v1/configurations",
            ),
            (
                Command::Configurations { limit: Some(100) },
                "/api/v1/configurations?limit=100",
            ),
            (
                Command::Node {
                    configuration_id: "configuration with space".to_owned(),
                    node_id: "узел&one".to_owned(),
                },
                "/api/v1/graph/node?configuration_id=configuration%20with%20space&node_id=%D1%83%D0%B7%D0%B5%D0%BB%26one",
            ),
            (
                Command::Relations {
                    configuration_id: "configuration".to_owned(),
                    node_id: "node".to_owned(),
                    direction: Direction::Incoming,
                    edge_kind: Some("depends_on".to_owned()),
                    limit: Some(1),
                },
                "/api/v1/graph/relations?configuration_id=configuration&node_id=node&direction=incoming&edge_kind=depends_on&limit=1",
            ),
            (
                Command::Traverse {
                    configuration_id: "configuration".to_owned(),
                    node_id: "node".to_owned(),
                    direction: Direction::Outgoing,
                    max_depth: 4,
                    edge_kind: Some("contains".to_owned()),
                    include_start: true,
                    limit: Some(50),
                },
                "/api/v1/graph/traverse?configuration_id=configuration&node_id=node&direction=outgoing&max_depth=4&edge_kind=contains&include_start=true&limit=50",
            ),
        ];
        for (command, expected) in cases {
            assert_eq!(request_target(&command), expected);
        }
    }

    #[test]
    fn request_headers_preserve_ipv4_and_ipv6_socket_addresses() {
        for address in [
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000),
            SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 4317),
        ] {
            let request = ClientRequest::new(address, Command::Health(HealthProbe::Live));
            let bytes = request_bytes(&request, "/health/live");
            assert_eq!(
                String::from_utf8(bytes).expect("request must be UTF-8"),
                format!(
                    "GET /health/live HTTP/1.1\r\nHost: {address}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
                )
            );
        }
    }

    #[test]
    fn response_parser_accepts_closed_http_versions_headers_and_statuses() {
        for response in [
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 11\r\n\r\n{\"ok\":true}".as_slice(),
            b"HTTP/1.0 200 OK\r\ncontent-type: APPLICATION/JSON\r\nX-Evidence: stable\r\n\r\n{\"ok\":true}".as_slice(),
        ] {
            assert_eq!(
                parse_response(response),
                Ok(ExecutionOutcome::Success(br#"{"ok":true}"#.to_vec()))
            );
        }

        assert_eq!(
            parse_response(
                b"HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\n\r\n{\"error\":{}}"
            ),
            Ok(ExecutionOutcome::ServerFailure(br#"{"error":{}}"#.to_vec()))
        );
    }

    #[test]
    fn response_parser_rejects_every_malformed_protocol_boundary() {
        let oversized_head = format!(
            "HTTP/1.1 200 OK\r\nX-Large: {}\r\nContent-Type: application/json\r\n\r\n{{}}",
            "a".repeat(MAX_HEAD_BYTES)
        );
        let oversized_body = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
            "a".repeat(MAX_BODY_BYTES + 1)
        );
        let cases: Vec<&[u8]> = vec![
            b"",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n",
            b"HTTP/2 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
            b"HTTP/1.1 XX OK\r\nContent-Type: application/json\r\n\r\n{}",
            b"HTTP/1.1 099 OK\r\nContent-Type: application/json\r\n\r\n{}",
            b"HTTP/1.1 200 OK\r\nInvalid\r\nContent-Type: application/json\r\n\r\n{}",
            b"HTTP/1.1 200 OK\r\nBad Name: value\r\nContent-Type: application/json\r\n\r\n{}",
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: application/json\r\n\r\n2\r\n{}\r\n0\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Type: application/json\r\n\r\n{}",
            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Length: 2\r\nContent-Type: application/json\r\n\r\n{}",
            b"HTTP/1.1 200 OK\r\nContent-Length: x\r\nContent-Type: application/json\r\n\r\n{}",
            b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nContent-Type: application/json\r\n\r\n{}",
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n{}",
            b"HTTP/1.1 200 OK\r\n\r\n{}",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n\xff",
            oversized_head.as_bytes(),
            oversized_body.as_bytes(),
        ];
        for response in cases {
            assert_eq!(
                parse_response(response),
                Err(ClientFailure::Protocol),
                "response prefix: {:?}",
                response.get(..response.len().min(80))
            );
        }
    }

    #[test]
    fn executor_sends_exact_request_and_preserves_success_and_server_bodies() {
        for (status, body, expected) in [
            (
                "200 OK",
                br#"{"status":"alive"}"#.as_slice(),
                ExecutionOutcome::Success(br#"{"status":"alive"}"#.to_vec()),
            ),
            (
                "503 Service Unavailable",
                br#"{"status":"not_ready"}"#.as_slice(),
                ExecutionOutcome::ServerFailure(br#"{"status":"not_ready"}"#.to_vec()),
            ),
        ] {
            let listener =
                TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("controlled listener must bind");
            let address = listener
                .local_addr()
                .expect("controlled address must be readable");
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let body = body.to_vec();
            let server = thread::spawn(move || {
                let (mut stream, _) = listener.accept().expect("client must connect");
                let mut bytes = Vec::new();
                let mut buffer = [0_u8; 1024];
                loop {
                    let read = stream.read(&mut buffer).expect("request must read");
                    bytes.extend_from_slice(&buffer[..read]);
                    if read == 0 || find_head_end(&bytes).is_some() {
                        break;
                    }
                }
                stream
                    .write_all(response.as_bytes())
                    .and_then(|()| stream.write_all(&body))
                    .expect("response must write");
                bytes
            });

            let request = ClientRequest::new(address, Command::Health(HealthProbe::Live));
            assert_eq!(HttpExecutor.execute(&request), expected);
            assert_eq!(
                server.join().expect("controlled server must join"),
                format!(
                    "GET /health/live HTTP/1.1\r\nHost: {address}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
                )
                .into_bytes()
            );
        }
    }

    #[test]
    fn executor_classifies_unreachable_and_malformed_responses_and_repeats() {
        let unreachable = ClientRequest::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
            Command::Health(HealthProbe::Live),
        );
        assert_eq!(
            HttpExecutor.execute(&unreachable),
            ExecutionOutcome::TransportFailure
        );

        for _ in 0..2 {
            let listener =
                TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("controlled listener must bind");
            let address = listener.local_addr().expect("address must be readable");
            let server = thread::spawn(move || {
                let (mut stream, _) = listener.accept().expect("client must connect");
                let mut request = Vec::new();
                let mut buffer = [0_u8; 1024];
                while find_head_end(&request).is_none() {
                    let read = stream.read(&mut buffer).expect("request must read");
                    assert_ne!(read, 0, "request must contain a complete head");
                    request.extend_from_slice(&buffer[..read]);
                }
                stream
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\ninvalid")
                    .expect("response must write");
            });
            let request = ClientRequest::new(address, Command::Health(HealthProbe::Ready));
            assert_eq!(
                HttpExecutor.execute(&request),
                ExecutionOutcome::ProtocolFailure
            );
            server.join().expect("controlled server must join");
        }
    }
}
