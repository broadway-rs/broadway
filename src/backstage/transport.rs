use std::net::IpAddr;

pub struct Transport{
    local: Location,
}

impl Transport{
    /// Create a new transport that listens with the specified address
    pub fn new(local: Location) -> Self{
        Self{
            local
        }
    }

    pub fn get_local(&self) -> &Location{
        &self.local
    }
}

#[derive(Clone)]
pub enum Location{
    Ip(IpAddr),
    URI(String),
}