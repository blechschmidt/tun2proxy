use crate::{
    error::Error, http::HttpManager, socks::SocksManager, socks::SocksVersion,
    tun2proxy::TunToProxy,
};
use socks5_impl::protocol::UserKey;
use std::net::{SocketAddr, ToSocketAddrs};

mod android;
pub mod error;
mod http;
pub mod setup;
mod socks;
mod tun2proxy;
mod virtdevice;
mod virtdns;

#[derive(Clone, Debug)]
pub struct Proxy {
    pub proxy_type: ProxyType,
    pub addr: SocketAddr,
    pub credentials: Option<UserKey>,
}

pub enum NetworkInterface {
    Named(String),
    Fd(std::os::fd::RawFd),
}

impl Proxy {
    pub fn from_url(s: &str) -> Result<Proxy, Error> {
        let e = format!("`{s}` is not a valid proxy URL");
        let url = url::Url::parse(s).map_err(|_| Error::from(&e))?;
        let e = format!("`{s}` does not contain a host");
        let host = url.host_str().ok_or(Error::from(e))?;

        let mut url_host = String::from(host);
        let e = format!("`{s}` does not contain a port");
        let port = url.port().ok_or(Error::from(&e))?;
        url_host.push(':');
        url_host.push_str(port.to_string().as_str());

        let e = format!("`{host}` could not be resolved");
        let mut addr_iter = url_host.to_socket_addrs().map_err(|_| Error::from(&e))?;

        let e = format!("`{host}` does not resolve to a usable IP address");
        let addr = addr_iter.next().ok_or(Error::from(&e))?;

        let credentials = if url.username() == "" && url.password().is_none() {
            None
        } else {
            let username = String::from(url.username());
            let password = String::from(url.password().unwrap_or(""));
            Some(UserKey::new(username, password))
        };

        let scheme = url.scheme();

        let proxy_type = match url.scheme().to_ascii_lowercase().as_str() {
            "socks4" => Some(ProxyType::Socks4),
            "socks5" => Some(ProxyType::Socks5),
            "http" => Some(ProxyType::Http),
            _ => None,
        }
        .ok_or(Error::from(&format!("`{scheme}` is an invalid proxy type")))?;

        Ok(Proxy {
            proxy_type,
            addr,
            credentials,
        })
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ProxyType {
    Socks4,
    Socks5,
    Http,
}

impl std::fmt::Display for ProxyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyType::Socks4 => write!(f, "socks4"),
            ProxyType::Socks5 => write!(f, "socks5"),
            ProxyType::Http => write!(f, "http"),
        }
    }
}

#[derive(Default)]
pub struct Options {
    virtdns: Option<virtdns::VirtualDns>,
    mtu: Option<usize>,
}

impl Options {
    pub fn new() -> Self {
        Options::default()
    }

    pub fn with_virtual_dns(mut self) -> Self {
        self.virtdns = Some(virtdns::VirtualDns::new());
        self
    }

    pub fn with_mtu(mut self, mtu: usize) -> Self {
        self.mtu = Some(mtu);
        self
    }
}

pub fn tun_to_proxy<'a>(
    interface: &NetworkInterface,
    proxy: &Proxy,
    options: Options,
) -> Result<TunToProxy<'a>, Error> {
    let mut ttp = TunToProxy::new(interface, options)?;
    match proxy.proxy_type {
        ProxyType::Socks4 => {
            ttp.add_connection_manager(SocksManager::new(
                proxy.addr,
                SocksVersion::V4,
                proxy.credentials.clone(),
            ));
        }
        ProxyType::Socks5 => {
            ttp.add_connection_manager(SocksManager::new(
                proxy.addr,
                SocksVersion::V5,
                proxy.credentials.clone(),
            ));
        }
        ProxyType::Http => {
            ttp.add_connection_manager(HttpManager::new(proxy.addr, proxy.credentials.clone()));
        }
    }
    Ok(ttp)
}

pub fn main_entry(
    interface: &NetworkInterface,
    proxy: &Proxy,
    options: Options,
) -> Result<(), Error> {
    let ttp = tun_to_proxy(interface, proxy, options);
    ttp?.run()
}
