use std::{
    io::{self, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::{Arc, OnceLock},
    time::Duration,
};

use rustls::{ClientConfig, ClientConnection, OwnedTrustAnchor, RootCertStore, ServerName};
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;
use url::Url;

#[derive(Debug, Clone)]
pub struct BlockingHttpClient {
    timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct RequestBuilder {
    client: BlockingHttpClient,
    method: String,
    url: Url,
    headers: Vec<(String, String)>,
}

#[derive(Debug)]
pub struct Response {
    status: u16,
    body: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("{0}")]
    Transport(String),
    #[error("unexpected HTTP {0}")]
    Status(u16, Response),
}

impl BlockingHttpClient {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    pub fn request(&self, method: &str, url: &str) -> Result<RequestBuilder, RequestError> {
        let url = Url::parse(url).map_err(|error| {
            RequestError::Transport(format!("invalid request URL `{url}`: {error}"))
        })?;
        Ok(RequestBuilder {
            client: self.clone(),
            method: method.to_owned(),
            url,
            headers: Vec::new(),
        })
    }
}

impl RequestBuilder {
    pub fn set(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }

    pub fn call(self) -> Result<Response, RequestError> {
        self.send_inner(&[], None)
    }

    pub fn send_bytes(self, body: &[u8]) -> Result<Response, RequestError> {
        self.send_inner(body, None)
    }

    pub fn send_json<T>(self, body: T) -> Result<Response, RequestError>
    where
        T: Serialize,
    {
        let body = serde_json::to_vec(&body).map_err(|error| {
            RequestError::Transport(format!("failed to serialize JSON request body: {error}"))
        })?;
        self.send_inner(&body, Some("application/json"))
    }

    fn send_inner(self, body: &[u8], content_type: Option<&str>) -> Result<Response, RequestError> {
        validate_http_token("HTTP method", &self.method)?;
        for (name, value) in &self.headers {
            validate_http_token("HTTP header name", name)?;
            validate_header_value(name, value)?;
        }

        let host = self.url.host_str().ok_or_else(|| {
            RequestError::Transport(format!("request URL `{}` is missing a host", self.url))
        })?;
        let port = self.url.port_or_known_default().ok_or_else(|| {
            RequestError::Transport(format!("request URL `{}` is missing a port", self.url))
        })?;
        let address = format!("{host}:{port}");

        let socket_addr = address
            .to_socket_addrs()
            .map_err(|error| {
                RequestError::Transport(format!("failed to resolve `{address}`: {error}"))
            })?
            .next()
            .ok_or_else(|| {
                RequestError::Transport(format!("no socket addresses resolved for `{address}`"))
            })?;
        let stream =
            TcpStream::connect_timeout(&socket_addr, self.client.timeout).map_err(|error| {
                RequestError::Transport(format!("failed to connect to `{address}`: {error}"))
            })?;
        stream
            .set_read_timeout(Some(self.client.timeout))
            .map_err(|error| {
                RequestError::Transport(format!(
                    "failed to set read timeout for `{address}`: {error}"
                ))
            })?;
        stream
            .set_write_timeout(Some(self.client.timeout))
            .map_err(|error| {
                RequestError::Transport(format!(
                    "failed to set write timeout for `{address}`: {error}"
                ))
            })?;

        let path = match self.url.query() {
            Some(query) => format!("{}?{query}", self.url.path()),
            None => self.url.path().to_owned(),
        };
        let host_header = match self.url.port() {
            Some(port) => format!("{host}:{port}"),
            None => host.to_owned(),
        };

        let mut request = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nContent-Length: {}\r\n",
            self.method,
            path,
            host_header,
            body.len()
        );
        if let Some(content_type) = content_type {
            request.push_str(&format!("Content-Type: {content_type}\r\n"));
        }
        for (name, value) in &self.headers {
            request.push_str(&format!("{name}: {value}\r\n"));
        }
        request.push_str("\r\n");

        let response = match self.url.scheme() {
            "http" => perform_request(stream, request.as_bytes(), body),
            "https" => {
                let server_name = ServerName::try_from(host).map_err(|error| {
                    RequestError::Transport(format!("invalid TLS server name `{host}`: {error}"))
                })?;
                let connection =
                    ClientConnection::new(tls_config().clone(), server_name).map_err(|error| {
                        RequestError::Transport(format!(
                            "failed to configure TLS connection for `{host}`: {error}"
                        ))
                    })?;
                let tls_stream = rustls::StreamOwned::new(connection, stream);
                perform_request(tls_stream, request.as_bytes(), body)
            }
            scheme => Err(RequestError::Transport(format!(
                "unsupported URL scheme `{scheme}` for `{}`",
                self.url
            ))),
        }?;

        if (200..=299).contains(&response.status) {
            Ok(response)
        } else {
            Err(RequestError::Status(response.status, response))
        }
    }
}

impl Response {
    pub fn into_json<T>(self) -> Result<T, serde_json::Error>
    where
        T: DeserializeOwned,
    {
        serde_json::from_slice(&self.body)
    }

    pub fn into_string(self) -> io::Result<String> {
        String::from_utf8(self.body)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.body
    }

    pub fn into_sanitized_error_body(self) -> Option<String> {
        sanitize_error_body(&String::from_utf8_lossy(&self.body))
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.body))
    }
}

fn perform_request<T>(
    mut stream: T,
    request_head: &[u8],
    body: &[u8],
) -> Result<Response, RequestError>
where
    T: Read + Write,
{
    stream.write_all(request_head).map_err(|error| {
        RequestError::Transport(format!("failed to write HTTP request head: {error}"))
    })?;
    if !body.is_empty() {
        stream.write_all(body).map_err(|error| {
            RequestError::Transport(format!("failed to write HTTP request body: {error}"))
        })?;
    }
    stream.flush().map_err(|error| {
        RequestError::Transport(format!("failed to flush HTTP request: {error}"))
    })?;

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).map_err(|error| {
        RequestError::Transport(format!("failed to read HTTP response: {error}"))
    })?;
    parse_response(raw).map_err(RequestError::Transport)
}

fn parse_response(raw: Vec<u8>) -> Result<Response, String> {
    let header_end = find_sequence(&raw, b"\r\n\r\n")
        .ok_or_else(|| "HTTP response did not include a header terminator".to_owned())?;
    let header_bytes = &raw[..header_end];
    let body_bytes = &raw[header_end + 4..];
    let header_text = String::from_utf8(header_bytes.to_vec())
        .map_err(|error| format!("HTTP response headers were not valid UTF-8: {error}"))?;

    let mut lines = header_text.split("\r\n");
    let status_line = lines
        .next()
        .ok_or_else(|| "HTTP response did not include a status line".to_owned())?;
    let mut status_parts = status_line.split_whitespace();
    let _http_version = status_parts
        .next()
        .ok_or_else(|| format!("invalid HTTP status line `{status_line}`"))?;
    let status = status_parts
        .next()
        .ok_or_else(|| format!("invalid HTTP status line `{status_line}`"))?
        .parse::<u16>()
        .map_err(|error| format!("invalid HTTP status code in `{status_line}`: {error}"))?;

    let mut transfer_encoding_chunked = false;
    let mut content_length = None;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim();
        let value = value.trim();
        if name.eq_ignore_ascii_case("transfer-encoding")
            && value
                .split(',')
                .any(|part| part.trim().eq_ignore_ascii_case("chunked"))
        {
            transfer_encoding_chunked = true;
        } else if name.eq_ignore_ascii_case("content-length") {
            content_length = Some(
                value
                    .parse::<usize>()
                    .map_err(|error| format!("invalid Content-Length `{value}`: {error}"))?,
            );
        }
    }

    let body = if transfer_encoding_chunked {
        decode_chunked_body(body_bytes)?
    } else if let Some(expected_len) = content_length {
        if body_bytes.len() < expected_len {
            return Err(format!(
                "HTTP response body was truncated: expected {expected_len} bytes, got {}",
                body_bytes.len()
            ));
        }
        body_bytes[..expected_len].to_vec()
    } else {
        body_bytes.to_vec()
    };

    Ok(Response { status, body })
}

fn decode_chunked_body(body: &[u8]) -> Result<Vec<u8>, String> {
    let mut cursor = 0;
    let mut decoded = Vec::new();

    loop {
        let line_end_relative = find_sequence(&body[cursor..], b"\r\n")
            .ok_or_else(|| "chunked HTTP response was missing a size delimiter".to_owned())?;
        let line_end = cursor + line_end_relative;
        let size_line = std::str::from_utf8(&body[cursor..line_end])
            .map_err(|error| format!("chunk size line was not valid UTF-8: {error}"))?;
        let size_hex = size_line
            .split(';')
            .next()
            .ok_or_else(|| "chunk size line was empty".to_owned())?
            .trim();
        let size = usize::from_str_radix(size_hex, 16)
            .map_err(|error| format!("invalid chunk size `{size_hex}`: {error}"))?;
        cursor = line_end + 2;

        if size == 0 {
            break;
        }

        let chunk_end = cursor + size;
        if body.len() < chunk_end + 2 {
            return Err("chunked HTTP response body was truncated".to_owned());
        }
        decoded.extend_from_slice(&body[cursor..chunk_end]);
        if &body[chunk_end..chunk_end + 2] != b"\r\n" {
            return Err("chunked HTTP response chunk was missing a CRLF terminator".to_owned());
        }
        cursor = chunk_end + 2;
    }

    Ok(decoded)
}

fn find_sequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

const MAX_ERROR_BODY_CHARS: usize = 512;
const REDACTED_ERROR_BODY: &str = "[redacted response body]";
const SENSITIVE_ERROR_BODY_MARKERS: [&str; 10] = [
    "access_token",
    "api_key",
    "authorization",
    "credential",
    "credential=",
    "password",
    "secret",
    "signature",
    "token",
    "x-amz-security-token",
];

fn sanitize_error_body(body: &str) -> Option<String> {
    let normalized = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return None;
    }

    let lower = normalized.to_ascii_lowercase();
    if SENSITIVE_ERROR_BODY_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Some(REDACTED_ERROR_BODY.to_owned());
    }

    Some(truncate_error_body(&normalized))
}

fn truncate_error_body(body: &str) -> String {
    let mut characters = body.chars();
    let truncated = characters
        .by_ref()
        .take(MAX_ERROR_BODY_CHARS)
        .collect::<String>();

    if characters.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

fn tls_config() -> &'static Arc<ClientConfig> {
    static TLS_CONFIG: OnceLock<Arc<ClientConfig>> = OnceLock::new();
    TLS_CONFIG.get_or_init(|| {
        let mut roots = RootCertStore::empty();
        roots.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|anchor| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                anchor.subject,
                anchor.spki,
                anchor.name_constraints,
            )
        }));

        Arc::new(
            ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(roots)
                .with_no_client_auth(),
        )
    })
}

fn validate_http_token(kind: &str, value: &str) -> Result<(), RequestError> {
    if value.is_empty() || !value.bytes().all(is_http_token_char) {
        return Err(RequestError::Transport(format!(
            "{kind} contains characters that are not allowed in HTTP tokens"
        )));
    }

    Ok(())
}

fn validate_header_value(name: &str, value: &str) -> Result<(), RequestError> {
    if value
        .bytes()
        .any(|byte| matches!(byte, b'\r' | b'\n' | 0x00..=0x08 | 0x0b..=0x1f | 0x7f))
    {
        return Err(RequestError::Transport(format!(
            "header `{name}` contains disallowed control characters"
        )));
    }

    Ok(())
}

fn is_http_token_char(byte: u8) -> bool {
    matches!(
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
            | b'0'..=b'9'
            | b'A'..=b'Z'
            | b'a'..=b'z'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_transport_error_contains(result: Result<Response, RequestError>, expected: &str) {
        match result {
            Err(RequestError::Transport(message)) => assert!(
                message.contains(expected),
                "expected `{message}` to contain `{expected}`"
            ),
            Err(other) => panic!("expected transport error, received {other:?}"),
            Ok(_) => panic!("expected request to fail validation before network I/O"),
        }
    }

    fn response_with_body(body: impl Into<Vec<u8>>) -> Response {
        Response {
            status: 500,
            body: body.into(),
        }
    }

    #[test]
    fn request_rejects_invalid_http_method_before_network_io() {
        let client = BlockingHttpClient::new(Duration::from_secs(1));
        let request = client
            .request("GET\r\nX-Injected: yes", "http://127.0.0.1:9/")
            .expect("test URL should parse");

        assert_transport_error_contains(request.call(), "HTTP method contains characters");
    }

    #[test]
    fn request_rejects_invalid_header_name_before_network_io() {
        let client = BlockingHttpClient::new(Duration::from_secs(1));
        let request = client
            .request("GET", "http://127.0.0.1:9/")
            .expect("test URL should parse")
            .set("Bad Header", "value");

        assert_transport_error_contains(request.call(), "HTTP header name contains characters");
    }

    #[test]
    fn request_rejects_header_value_line_breaks_before_network_io() {
        let client = BlockingHttpClient::new(Duration::from_secs(1));
        let request = client
            .request("GET", "http://127.0.0.1:9/")
            .expect("test URL should parse")
            .set("Authorization", "Bearer token\r\nX-Injected: yes");

        assert_transport_error_contains(
            request.call(),
            "header `Authorization` contains disallowed control characters",
        );
    }

    #[test]
    fn sanitized_error_body_redacts_sensitive_markers() {
        let response =
            response_with_body(br#"{"message":"bad token","access_token":"doc-secret"}"#.to_vec());

        assert_eq!(
            response.into_sanitized_error_body().as_deref(),
            Some(REDACTED_ERROR_BODY)
        );
    }

    #[test]
    fn sanitized_error_body_collapses_whitespace_for_safe_body() {
        let response = response_with_body("service\n  temporarily\tunavailable");

        assert_eq!(
            response.into_sanitized_error_body().as_deref(),
            Some("service temporarily unavailable")
        );
    }

    #[test]
    fn sanitized_error_body_truncates_safe_body() {
        let response = response_with_body("a".repeat(MAX_ERROR_BODY_CHARS + 8));
        let body = response
            .into_sanitized_error_body()
            .expect("non-empty safe body should remain available");

        assert_eq!(body.chars().count(), MAX_ERROR_BODY_CHARS + 3);
        assert!(body.ends_with("..."));
    }

    #[test]
    fn sanitized_error_body_omits_blank_body() {
        let response = response_with_body(" \n\t ");

        assert_eq!(response.into_sanitized_error_body(), None);
    }
}
