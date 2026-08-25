use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::mpsc,
    task::JoinHandle,
};

const MAX_CAPTURED_REQUEST_BYTES: usize = 1_024 * 1_024;

pub(crate) struct ControlledServer {
    base_url: String,
    requests: mpsc::Receiver<Vec<u8>>,
    task: JoinHandle<()>,
}

impl ControlledServer {
    pub(crate) async fn spawn(responses: Vec<Option<Vec<u8>>>) -> Self {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("controlled listener must bind");
        let address = listener.local_addr().expect("listener address must exist");
        let (sender, requests) = mpsc::channel(responses.len().max(1));
        let task = tokio::spawn(async move {
            for response in responses {
                let (mut stream, _) = listener.accept().await.expect("request must connect");
                let request = read_request(&mut stream).await;
                sender
                    .send(request)
                    .await
                    .expect("test must receive request");
                if let Some(response) = response {
                    stream
                        .write_all(&response)
                        .await
                        .expect("response must write");
                    stream.shutdown().await.expect("response must close");
                } else {
                    let mut remainder = Vec::new();
                    stream
                        .read_to_end(&mut remainder)
                        .await
                        .expect("cancelled client must close connection");
                }
            }
        });
        Self {
            base_url: format!("http://{address}"),
            requests,
            task,
        }
    }

    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    pub(crate) async fn next_request(&mut self) -> Vec<u8> {
        self.requests
            .recv()
            .await
            .expect("request must be captured")
    }

    pub(crate) async fn finish(self) {
        tokio::time::timeout(std::time::Duration::from_secs(2), self.task)
            .await
            .expect("controlled server must stop without leaked work")
            .expect("controlled server task must succeed");
    }
}

async fn read_request(stream: &mut tokio::net::TcpStream) -> Vec<u8> {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 4_096];
    let header_end = loop {
        let read = stream.read(&mut buffer).await.expect("request must read");
        assert!(read > 0, "request ended before headers");
        request.extend_from_slice(&buffer[..read]);
        assert!(
            request.len() <= MAX_CAPTURED_REQUEST_BYTES,
            "captured request exceeded test bound"
        );
        if let Some(position) = request.windows(4).position(|window| window == b"\r\n\r\n") {
            break position + 4;
        }
    };
    let content_length = parse_content_length(&request[..header_end]);
    while request.len() < header_end + content_length {
        let read = stream
            .read(&mut buffer)
            .await
            .expect("request body must read");
        assert!(read > 0, "request ended before body");
        request.extend_from_slice(&buffer[..read]);
        assert!(
            request.len() <= MAX_CAPTURED_REQUEST_BYTES,
            "captured request exceeded test bound"
        );
    }
    request.truncate(header_end + content_length);
    request
}

fn parse_content_length(headers: &[u8]) -> usize {
    let headers = std::str::from_utf8(headers).expect("request headers must be UTF-8");
    headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length").then(|| {
                value
                    .trim()
                    .parse()
                    .expect("content length must be numeric")
            })
        })
        .unwrap_or(0)
}

pub(crate) fn http_response(status: u16, body: &[u8]) -> Vec<u8> {
    raw_http_response(
        &format!("{status} Test"),
        &[&format!("Content-Length: {}", body.len())],
        body,
    )
}

pub(crate) fn raw_http_response(status: &str, headers: &[&str], body: &[u8]) -> Vec<u8> {
    let mut response = format!("HTTP/1.1 {status}\r\nConnection: close\r\n");
    for header in headers {
        response.push_str(header);
        response.push_str("\r\n");
    }
    response.push_str("\r\n");
    let mut bytes = response.into_bytes();
    bytes.extend_from_slice(body);
    bytes
}
