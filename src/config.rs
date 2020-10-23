#[derive(Clone)]
pub struct ServerConfiguration {
    pub port: u16,
    pub name: String,
    pub max_clients: usize,
}

impl ServerConfiguration {
    pub fn load() -> ServerConfiguration {
        ServerConfiguration {
            port: 8100,
            name: String::from("A test server"),
            max_clients: 6,
        }
    }
}
