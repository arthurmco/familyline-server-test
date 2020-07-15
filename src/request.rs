use regex::Regex;

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

                        match accept_formats
                            .split(',')
                            .find(|fmt| fmt.contains("application/json") || fmt.contains("text/html"))
                        {
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
