/*
    This module provides a simple URL parser and HTTP client.
    This uses https://browser.engineering/http.html to help me and is the guide that im following to learn browser engeineering.
    I have done it in rust, and the way I tend to program, for good or bad, who knows!
    There are some differences from the guide, for example I have structs and enums to make it easier to work with URLs and responses.
    I also used more functions to make it easier to read and understand.

    Tasks to complete:
    - [ ] Keep-Alive
    - [ ] Caching
    - [ ] Compression
*/

use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    sync::Arc,
};

use rustls::RootCertStore;
use socket2::{Domain, Protocol, Socket, Type};

use crate::{
    layout::text::{Body, show},
    renderer::init_renderer,
};

pub fn load(url: &str) -> Result<(), String> {
    let mut url = URL::from_string(url)?;
    let mut response = url
        .clone()
        .request()
        .map_err(|e| format!("Failed to load URL: {}", e))?;

    // Redirect handling
    let mut limit = 10; // Prevent infinite redirects
    while (response.get_response_code() <= Some(200) || response.get_response_code() >= Some(300))
        && limit > 0
    {
        match response.get_response_code() {
            Some(code) if code >= 300 && code < 400 => {
                let redirect_url = response
                    .headers
                    .get("Location")
                    .ok_or_else(|| "No Location header found for redirect".to_string())?;
                // If is part of the same domain, we can just change the path
                if !redirect_url.contains("://") {
                    url.path = redirect_url.to_string();
                } else {
                    url = URL::from_string(
                        response
                            .headers
                            .get("Location")
                            .ok_or_else(|| "No Location header found for redirect".to_string())?,
                    )
                    .map_err(|e| format!("Failed to parse redirect URL: {}", e))?;
                }
            }
            Some(_) => break,
            None => break,
        }
        response = url
            .clone()
            .request()
            .map_err(|e| format!("Failed to load URL after redirect: {}", e))?;
        limit -= 1;
    }

    if response.get_response_code() <= Some(200) || response.get_response_code() > Some(300) {
        // Cache response
    }

    if url.show_source {
        response.display_source();
    } else {
        response.display();
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct URL {
    scheme: Scheme,
    pub host: String,
    pub path: String,
    pub queries: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub port: u16,
    method: Method,
    show_source: bool,
}

impl URL {
    pub fn request(self) -> Result<Response, String> {
        match &self.scheme {
            Scheme::Http => self.request_http(),
            Scheme::Https => self.request_https(),
            Scheme::File => self.request_file(),
            Scheme::Data(s) => URL::request_data(s.to_string()),
            _ => self.request_blank(),
        }
    }
    fn request_http(&self) -> Result<Response, String> {
        let s = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))
            .map_err(|e| format!("Failed to create socket: {}", e))?;

        let address: SocketAddr = format!("{}:{}", self.host, self.port)
            .to_socket_addrs()
            .map_err(|e| format!("Failed to resolve host: {}", e))?
            .next()
            .ok_or_else(|| format!("No address found for host: {}", self.host))?;

        s.connect(&address.into())
            .map_err(|e| format!("Failed to connect to {} / {:#?}: {}", self.host, address, e))?;

        // TODO: http://browser.engineering/http.html 1-6 keep-alive
        let request = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nAccept-Encoding: identity\r\n\r\n",
            self.method.as_str(),
            self.path,
            self.host
        );

        s.send(request.as_bytes())
            .map_err(|e| format!("Failed to send request: {}", e))?;

        let mut response = Vec::new();
        let mut buf = [std::mem::MaybeUninit::<u8>::uninit(); 8192];
        loop {
            match s.recv(&mut buf) {
                Ok(0) => break, // Connection closed
                Ok(n) => {
                    let bytes = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, n) };
                    response.extend_from_slice(bytes);
                    if n == 0 {
                        break; // No more data
                    }
                }
                Err(e) => return Err(format!("Failed to read response: {}", e)),
            }
        }
        let response_str = String::from_utf8_lossy(&response).to_string();
        Ok(Response::from_string(&response_str)
            .map_err(|e| format!("Failed to parse response: {}", e))?)
    }

    fn request_https(&self) -> Result<Response, String> {
        let root_store = RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.into(),
        };
        let mut config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        // TODO: http://browser.engineering/http.html 1-6 keep-alive
        let request = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nAccept-Encoding: identity\r\n\r\n",
            self.method.as_str(),
            self.path,
            self.host
        );

        // Allow using SSLKEYLOGFILE.
        config.key_log = Arc::new(rustls::KeyLogFile::new());

        let mut conn =
            rustls::ClientConnection::new(Arc::new(config), self.host.clone().try_into().unwrap())
                .unwrap();
        let mut sock = TcpStream::connect(format!("{}:{}", self.host, self.port)).unwrap();
        let mut tls = rustls::Stream::new(&mut conn, &mut sock);

        tls.write_all(request.as_bytes())
            .map_err(|e| format!("Failed to write request: {}", e))?;

        let mut plaintext = Vec::new();
        tls.read_to_end(&mut plaintext).unwrap();

        let response_str = String::from_utf8_lossy(&plaintext).to_string();
        Ok(Response::from_string(&response_str)
            .map_err(|e| format!("Failed to parse response: {}", e))?)
    }

    fn request_file(&self) -> Result<Response, String> {
        let path = &self.path;
        let mut file = std::fs::File::open(path)
            .map_err(|e| format!("Failed to open file {}: {}", path, e))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read file {}: {}", path, e))?;
        Ok(Response::new(
            "200 OK".to_string(),
            HashMap::new(),
            Body::new(contents),
        ))
    }

    /// This handles data e.g:
    /// data:text/html,<html><head><title>Hello</title></head><body><h1>Hello, world!</h1><p>This is a <strong>bold</strong> paragraph with <em>italic</em> text and a <a href='https://example.com'>link</a>.</p><ul><li>List item 1</li><li>List item 2</li></ul></body></html>
    /// We currently only support text/html data.
    fn request_data(s: String) -> Result<Response, String> {
        s.split_once(',')
            .map(|(_, data)| {
                Response::new(
                    "200 OK".to_string(),
                    HashMap::new(),
                    Body::new(data.to_string()),
                )
            })
            .ok_or_else(|| "Invalid data URL format".to_string())
    }

    // This handles about:blank
    fn request_blank(&self) -> Result<Response, String> {
        // Handle about:blank or view-source: URLs
        if self.scheme == Scheme::AboutBlank {
            Ok(Response::new(
                "200 OK".to_string(),
                HashMap::new(),
                Body::new("<html><body></body></html>".to_string()),
            ))
        } else {
            Err("Unsupported URL scheme for request".to_string())
        }
    }
}

impl URL {
    fn new(
        scheme: &Scheme,
        host: impl std::fmt::Display,
        path: impl std::fmt::Display,
        port: u16,
        method: &Method,
        queries: &HashMap<String, String>,
        headers: &HashMap<String, String>,
        show_source: bool,
    ) -> Self {
        URL {
            scheme: scheme.clone(),
            host: host.to_string(),
            path: path.to_string(),
            port: port.clone(),
            method: method.clone(),
            queries: queries.clone(),
            headers: headers.clone(),
            show_source,
        }
    }

    pub fn from_string(url: impl std::fmt::Display) -> Result<Self, String> {
        let mut url = url.to_string();
        let show_source = url.starts_with("view-source:");
        if url.starts_with("view-source:") {
            url = url.replace("view-source:", "");
        }
        if url.is_empty() {
            Err("URL cannot be empty".to_string())
        } else {
            // Edge cases for testing urls
            if url.starts_with("data:") {
                return Ok(URL::new(
                    &Scheme::Data(url.split(':').nth(1).unwrap_or("").to_string()),
                    "",
                    "",
                    0,               // Port is not applicable for data URLs",
                    &Method::Get,    // Default method for HTTP
                    &HashMap::new(), // queries empty by default
                    &HashMap::new(), // headers empty by default
                    url.starts_with("view-source:"),
                ));
            }
            if url == "about:blank" {
                return Ok(URL::new(
                    &Scheme::AboutBlank,
                    "",
                    "",
                    0,               // Port is not applicable for about:blank
                    &Method::Get,    // Default method for HTTP
                    &HashMap::new(), // queries empty by default
                    &HashMap::new(), // headers empty by default
                    show_source,
                ));
            }
            let scheme = url
                .split("://")
                .next()
                .and_then(|scheme| Scheme::from_str(scheme))
                .ok_or_else(|| "Unsupported URL scheme".to_string())?;
            let host = url
                .split("://")
                .nth(1)
                .and_then(|rest| rest.split('/').next())
                .ok_or_else(|| "Invalid URL format".to_string())?
                .to_string();
            let mut path = url
                .split("://")
                .nth(1)
                .and_then(|rest| rest.splitn(2, '/').nth(1))
                .unwrap_or("/")
                .split('?')
                .next()
                .unwrap_or("/")
                .to_string();
            if !path.starts_with('/') {
                path.insert(0, '/');
            }
            let queries = url
                .split('?')
                .nth(1)
                .map(|q| {
                    q.split('&')
                        .filter_map(|pair| {
                            let mut split = pair.splitn(2, '=');
                            if let (Some(k), Some(v)) = (split.next(), split.next()) {
                                Some((k.to_string(), v.to_string()))
                            } else {
                                None
                            }
                        })
                        .collect::<HashMap<String, String>>()
                })
                .unwrap_or_else(HashMap::new);
            let port = if let Some(port_str) = host.split(':').nth(1) {
                port_str
                    .parse::<u16>()
                    .map_err(|_| "Invalid port number".to_string())?
            } else {
                match scheme {
                    Scheme::Http => 80,   // Default port for HTTP
                    Scheme::Https => 443, // Default port for HTTPS
                    _ => 0,               // Default to 0 for other schemes
                }
            };
            Ok(URL::new(
                &scheme,
                host,
                path,
                port,
                &Method::Get, // Default method for HTTP
                &queries,
                &HashMap::new(), // headers empty by default
                show_source,
            ))
        }
    }

    pub fn build(&self) -> String {
        let mut url = format!("{}://{}{}", self.scheme.as_str(), self.host, self.path);
        if !self.queries.is_empty() {
            url.push('?');
            url.push_str(
                &self
                    .queries
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&"),
            );
        }
        url
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: String,
    pub headers: HashMap<String, String>,
    pub body: Body,
}

impl Response {
    pub fn new(status: String, headers: HashMap<String, String>, body: Body) -> Self {
        Response {
            status,
            headers,
            body,
        }
    }

    pub fn from_string(response: &str) -> Result<Self, String> {
        let mut lines = response.lines();
        let status_line = lines.next().ok_or("Empty response")?;
        let status = status_line.to_string();
        let mut headers = HashMap::new();
        let mut body = String::new();
        let mut headers_done = false;

        for line in lines {
            if line.is_empty() {
                headers_done = true;
                continue; // Empty line indicates end of headers
            }
            if headers_done {
                body.push_str(line);
                continue; // Collect body after headers
            }
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Ok(Response::new(status, headers, Body::new(body)))
    }

    pub fn display(&self) {
        let _ = init_renderer(self.body.clone());
    }

    pub fn display_source(&self) {
        let _ = init_renderer(self.body.clone());
    }

    pub fn get_response_code(&self) -> Option<u16> {
        let status = self
            .status
            .split(" ")
            .nth(1)
            .and_then(|code| code.parse::<u16>().ok());
        return status;
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Method {
    Post,
    Put,
    Delete,
    Get,
}

impl Method {
    fn as_str(&self) -> &str {
        match self {
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Get => "GET",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum Scheme {
    Http,
    Https,
    File,
    Data(String), // Data URLs can have a specific type, e.g., "text/html"
    ViewSource,
    AboutBlank,
}

impl Scheme {
    fn as_str(&self) -> &str {
        match self {
            Scheme::Http => "http",
            Scheme::Https => "https",
            Scheme::File => "file",
            Scheme::Data(_) => "data",
            Scheme::ViewSource => "view-source",
            Scheme::AboutBlank => "about:blank",
        }
    }
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "http" => Some(Scheme::Http),
            "https" => Some(Scheme::Https),
            "file" => Some(Scheme::File),
            "data" => Some(Scheme::Data(String::new())), // Default to empty data type
            "view-source" => Some(Scheme::ViewSource),   // Default to empty data type
            "about:blank" => Some(Scheme::AboutBlank),
            _ => None,
        }
    }
}
