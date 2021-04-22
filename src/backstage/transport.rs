use async_std::net::IpAddr;

pub enum Location{
    Ip(IpAddr),
    URI(String),
}