// Network helper functions for UDP and heartbeat sockets.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use if_addrs::get_if_addrs;

use crate::app::UdpConfig;
use crate::constants::HEARTBEAT_PORT;

pub async fn bind_heartbeat_socket(bind_addr: IpAddr) -> std::io::Result<tokio::net::UdpSocket> {
    tokio::net::UdpSocket::bind(SocketAddr::new(bind_addr, 0)).await
}

pub fn resolve_local_ip_for_target(target: IpAddr) -> std::io::Result<IpAddr> {
    let socket = std::net::UdpSocket::bind(("0.0.0.0", 0))?;
    socket.connect(SocketAddr::new(target, HEARTBEAT_PORT))?;
    Ok(socket.local_addr()?.ip())
}

pub fn resolve_default_route_ip() -> std::io::Result<IpAddr> {
    let socket = std::net::UdpSocket::bind(("0.0.0.0", 0))?;
    socket.connect(("1.1.1.1", 80))?;
    Ok(socket.local_addr()?.ip())
}

pub fn fallback_local_ip(config: &UdpConfig) -> Option<IpAddr> {
    match config.bind_addr {
        IpAddr::V4(addr) if !addr.is_loopback() && !addr.is_unspecified() => Some(IpAddr::V4(addr)),
        _ => preferred_private_ipv4().or_else(|| resolve_default_route_ip().ok()),
    }
}

pub fn resolve_broadcast_bind_ip(config: &UdpConfig, pending_detect: bool) -> Option<IpAddr> {
    if !pending_detect {
        return None;
    }
    fallback_local_ip(config)
}

pub fn preferred_private_ipv4() -> Option<IpAddr> {
    let ifaces = get_if_addrs().ok()?;
    for iface in ifaces {
        if let if_addrs::IfAddr::V4(v4) = iface.addr {
            let ip = v4.ip;
            if is_private_ipv4(ip) && !ip.is_loopback() && !ip.is_link_local() {
                return Some(IpAddr::V4(ip));
            }
        }
    }
    None
}

pub fn is_private_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    match octets {
        [10, ..] => true,
        [172, second, ..] if (16..=31).contains(&second) => true,
        [192, 168, ..] => true,
        _ => false,
    }
}
