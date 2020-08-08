use chrono::prelude::*;
use pnet::datalink;
use regex::Regex;

use crate::server::ServerDiscoveryInfo;

/// Represents what was found in the http request
pub struct HTTPRequestInfo {
    // General headers
    pub url: String,
    pub method: String,
    pub format: Option<String>,
    pub auth: Option<String>,
    pub length: Option<usize>,

    // Body
    pub body: Option<String>,

    // Specific headers for the game server
    pub player_code: Option<String>,
}

impl HTTPRequestInfo {
    pub fn parse_http_request(s: String) -> Option<HTTPRequestInfo> {
        let mut request_url: Option<(String, String)> = None;
        let url_regex = Regex::new(r"^([A-Z]*)\s(.*)\sHTTP/(\d\.\d)").unwrap();

        let mut request_format: Option<String> = None;
        let accept_regex = Regex::new(r"Accept: (.*)").unwrap();

        let mut auth_format: Option<String> = None;
        let auth_regex = Regex::new(r"Authorization: Basic (.*)").unwrap();

        let mut code_format: Option<String> = None;
        let code_regex = Regex::new(r"X-Client-Code: (.*)").unwrap();

        let mut length_format: Option<usize> = None;
        let length_regex = Regex::new(r"Content-Length: (.*)").unwrap();

        let mut body = None;

        for line in s.split("\n") {
            // An empty line means that the next line is the body
            if line == "\r" || line == "" {
                body = Some("".to_string());
                continue;
            }

            if let Some(s) = body {
                let mut new_s = s.clone();
                let sline = format!("{}\n", line);
                new_s.push_str(&sline);
                body = Some(new_s);
            } else {
                // Check if this line has some kind of request url
                if request_url.is_none() {
                    if let Some(caps) = url_regex.captures(line) {
                        let method = caps.get(1).unwrap().as_str().to_string();
                        let url = caps.get(2).unwrap().as_str().to_string();
                        let version = caps.get(3).unwrap().as_str();

                        println!(
                            "request {} for url {} with http ver {}",
                            method, url, version
                        );

                        if version != "1.0" && version != "1.1" {
                            return None;
                        }

                        request_url = Some((method, url))
                    }
                }

                // Check the formats the client requested.
                // Only json will be supported, because server metadata is supported that way.
                // Maybe html (and, by extension, CSS?) will be supported one day.
                if request_format.is_none() {
                    if let Some(caps) = accept_regex.captures(line) {
                        let accept_formats = caps.get(1).unwrap().as_str();

                        match accept_formats.split(',').find(|fmt| {
                            fmt.contains("application/json") || fmt.contains("text/html")
                        }) {
                            Some(fmt) => {
                                request_format = Some(fmt.trim().to_string());
                            }
                            None => {}
                        }
                    }
                }

                // Check if we have some sort of authentication header.
                // It is not required, except to download maps.
                if auth_format.is_none() {
                    if let Some(caps) = auth_regex.captures(line) {
                        let auth_data = caps.get(1).unwrap().as_str();
                        auth_format = Some(auth_data.trim().to_string())
                    }
                }

                // Check for the client code
                // If not here, assume the client did not log on and
                // deny access to all endpoints but the login one
                if code_format.is_none() {
                    if let Some(caps) = code_regex.captures(line) {
                        let code_data = caps.get(1).unwrap().as_str();
                        code_format = Some(code_data.trim().to_string())
                    }
                }

                // Check for the content length
                //
                // Useful only if we have a body
                if length_format.is_none() {
                    if let Some(caps) = length_regex.captures(line) {
                        length_format = match caps.get(1).unwrap().as_str().parse() {
                            Ok(v) => Some(v),
                            Err(_) => None,
                        }
                    }
                }
            }
        }

        match request_url {
            None => None,
            Some((method, url)) => Some(HTTPRequestInfo {
                url,
                method,
                format: request_format,
                auth: auth_format,
                length: length_format,
                body: body,
                player_code: code_format,
            }),
        }
    }
}

pub struct HTTPResponse {
    pub result: String,

    pub keep_alive: bool,
}

fn format_rfc2616_date(date: DateTime<Utc>) -> String {
    date.format("%a, %d %b %Y %T GMT").to_string()
}

impl HTTPResponse {
    /// Creates the http response
    pub fn new(http_code: u32, content: &str, additional_headers: Vec<String>) -> HTTPResponse {
        let mut lines: Vec<String> = Vec::new();
        let now: DateTime<Utc> = Utc::now();

        let codestr = match http_code {
            101 => "101 Switching Protocols",
            200 => "200 OK",
            201 => "201 Created",
            400 => "400 Bad Request",
            401 => "401 Unauthorized",
            403 => "403 Forbidden",
            404 => "404 Not Found",
            415 => "415 Unsupported Media Type",
            500 => "500 Internal Server Error",
            503 => "503 Service Unavailable",
            505 => "505 HTTP Version Not Supported",
            _ => panic!("unknown http error!"),
        };

        let body = content.to_string();

        lines.push(format!("HTTP/1.1 {}", codestr));
        lines.push(format!("Date: {}", format_rfc2616_date(now)));
        lines.push("Cache-Control: private, max-age=0".to_string());
        lines.push("Server: familyline-server 0.0.1-test".to_string());

        if body.len() > 0 {
            lines.push(format!("Content-Length: {}", body.len()));
        }

        if http_code >= 200 && http_code <= 299 {
            lines.push("Content-Type: application/json".to_string());
        }

        lines.extend(additional_headers);

        lines.push("".to_string());

        lines.push(body);

        return HTTPResponse {
            result: lines.join("\r\n"),
            keep_alive: false,
        };
    }
}

/// Find local IPv4 address
/// No IPv6 support yet because I am pretty sure my router is shit
/// and will not work well with it.
/// (this is also the reason why I set up only an ipv4 multicast)
///
/// Used primarily for the discover message response we receive in
/// multicast to tell the client where to connect to
///
/// If no address is found, it falls back to the loopback address.
pub fn find_local_address() -> String {
    for iface in datalink::interfaces()
        .iter()
        .filter(|i| i.ips.len() > 0)
        .filter(|i| {
            if let Some(mac) = i.mac {
                !mac.is_zero()
            } else {
                false
            }
        })
    {
        let ips = &iface.ips;
        return ips
            .iter()
            .filter(|i| i.ip().is_ipv4())
            .map(|i| i.ip().to_string())
            .nth(0)
            .or_else(|| Some(String::from("127.0.0.1")))
            .unwrap();
    }

    return String::from("127.0.0.1");
}

// Parse a discover message, return a response to it
pub fn parse_discover_message(s: String, sinfo: &ServerDiscoveryInfo) -> Option<String> {
    enum DiscoverOperation {
        Search,
    };

    let mut op: Option<DiscoverOperation> = None;
    let mut valid_servicetype = false;
    let mut valid_method = false;

    for line in s.split("\r\n") {
        println!("> {}", line);

        if line.starts_with("M-SEARCH *") {
            op = Some(DiscoverOperation::Search);
        }

        if line == "MAN: \"ssdp:discover\"" {
            valid_method = true;
        }

        // Respond to ssdp:all too?
        if line == "ST: game_server:familyline1" {
            valid_servicetype = true;
        }

        // Message ended.
        if line.trim() == "" {
            break;
        }
    }

    if !valid_servicetype || !valid_method {
        return None;
    }

    let sinfo_str = serde_json::to_string(&sinfo).unwrap();

    let now: DateTime<Utc> = Utc::now();
    let response_lines = vec![
        String::from("HTTP/1.1 200 OK"),
        String::from("Cache-Control: max-age=60"),
        format!("Date: {}", format_rfc2616_date(now)),
        String::from("Ext: "),
        format!("Location: http://{}:6142", find_local_address()),
        format!("Content-Length: {}", sinfo_str.len()),
        String::from("Server: familyline-server 0.0.1-test"),
        String::from(""),
        sinfo_str,
    ];

    println!(" -- response is valid --");

    return Some(response_lines.join("\r\n"));
}
