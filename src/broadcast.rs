/// Code responsible for sending a broadcast message to all
/// clients telling where the server is.
///
/// This will allow the server to show up in a client machine, assuming
/// they are both in the same network.
use pnet::datalink;

use crate::config::ServerConfiguration;
use crate::messages::{send_client_count_message, FMessage};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::mpsc::Sender;

use vecfold::VecFoldResult;

fn format_rfc2616_date(date: DateTime<Utc>) -> String {
    date.format("%a, %d %b %Y %T GMT").to_string()
}

/// Since UDP is unreliable, we might need to send the response
/// packet more than once, because it will not give an error even
/// if the packet did not reach its destination.
async fn send_multiple(
    socket: &mut UdpSocket,
    addr: &SocketAddr,
    buf: String,
    count: usize,
) -> Result<usize, std::io::Error> {
    let data = buf.as_bytes();
    let mut v = vec![];

    for _i in 0..count {
        v.push(socket.send_to(&data, addr).await);
    }

    match v.foldr() {
        Ok(sizes) => Ok(**sizes.iter().max().unwrap()),
        Err(e) => Err(std::io::Error::from_raw_os_error(e.raw_os_error().unwrap())),
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

#[derive(Debug, Deserialize, Serialize)]
struct DiscoveryResponse {
    name: String,
    port: u16,
    max_clients: usize,
    num_clients: usize,
}

async fn create_discovery_response(
    c: &ServerConfiguration,
    sender: &Sender<FMessage>,
) -> DiscoveryResponse {
    let mut sender = sender.clone();

    match send_client_count_message(&mut sender).await {
        Ok(num_clients) => DiscoveryResponse {
            name: c.name.clone(),
            port: c.port,
            max_clients: c.max_clients,
            num_clients,
        },
        Err(_) => panic!("Unexpected message"),
    }
}

// Parse a discover message, return a response to it
fn parse_discover_message(s: String, sres: DiscoveryResponse) -> Option<String> {
    enum DiscoverOperation {
        Search,
    };

    let mut op: Option<DiscoverOperation> = None;
    let mut valid_servicetype = false;
    let mut valid_method = false;

    for line in s.split("\r\n") {
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

    let sinfo_str = serde_json::to_string(&sres).unwrap();

    let now: DateTime<Utc> = Utc::now();
    let response_lines = vec![
        String::from("HTTP/1.1 200 OK"),
        String::from("Cache-Control: max-age=60"),
        format!("Date: {}", format_rfc2616_date(now)),
        String::from("Ext: "),
        format!("Location: http://{}:{}", find_local_address(), sres.port),
        format!("Content-Length: {}", sinfo_str.len()),
        String::from("Server: familyline-server 0.0.1-test"),
        String::from(""),
        sinfo_str,
    ];

    return Some(response_lines.join("\r\n"));
}

pub async fn run_discovery_thread(config: &ServerConfiguration, sender: Sender<FMessage>) {
    let config = config.clone();
    let laddr = find_local_address();
    let mut udp_discover = UdpSocket::bind(format!("{}:1983", laddr)).await.unwrap();
    tokio::spawn(async move {
        udp_discover
            .join_multicast_v4("239.255.255.250".parse().unwrap(), laddr.parse().unwrap())
            .unwrap();
        println!("Server is discoverable!");
        loop {
            let mut buf = vec![0; 1024];
            match udp_discover.recv_from(&mut buf).await {
                Ok((msize, sockaddr)) => {
                    let msg = match String::from_utf8(buf) {
                        Ok(m) => m,
                        Err(e) => {
                            format!("{:?} ({:?})", e.as_bytes(), e.utf8_error());
                            "".to_string()
                        }
                    };

                    let res = parse_discover_message(
                        msg,
                        create_discovery_response(&config, &sender).await,
                    );

                    if let Some(response) = res {
                        match send_multiple(&mut udp_discover, &sockaddr, response.clone(), 2).await
                        {
                            Ok(ssize) => {
                                println!("{:?}, {} bytes", sockaddr, ssize);
                            }
                            Err(e) => {
                                eprintln!("{:?}", e);
                            }
                        }
                    }
                }
                Err(_) => {
                    eprintln!("An error happened while receiving a message on the listen socket")
                }
            }
        }
    });
}
