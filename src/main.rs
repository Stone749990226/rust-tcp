use std::{
    collections::{HashMap, hash_map::Entry},
    io,
    net::Ipv4Addr,
};

use etherparse::IpNumber;

mod tcp;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}
fn main() -> io::Result<()> {
    let mut connections: HashMap<Quad, tcp::Connection> = Default::default();
    let nic = tun_tap::Iface::without_packet_info("tun0", tun_tap::Mode::Tun)?;
    let mut buf = [0u8; 1504];
    loop {
        let nbytes = nic.recv(&mut buf[..])?;
        match etherparse::Ipv4HeaderSlice::from_slice(&buf[..nbytes]) {
            Ok(ip_hdr) => {
                let src = ip_hdr.source_addr();
                let dst = ip_hdr.destination_addr();
                if ip_hdr.protocol() != IpNumber::TCP {
                    continue;
                }
                match etherparse::TcpHeaderSlice::from_slice(&buf[ip_hdr.slice().len()..nbytes]) {
                    Ok(tcp_hdr) => {
                        let data_idx = ip_hdr.slice().len() + tcp_hdr.slice().len();
                        match connections.entry(Quad {
                            src: (src, tcp_hdr.source_port()),
                            dst: (dst, tcp_hdr.destination_port()),
                        }) {
                            Entry::Occupied(mut c) => c.get_mut().on_packet(
                                &nic,
                                ip_hdr,
                                tcp_hdr,
                                &buf[data_idx..nbytes],
                            )?,
                            Entry::Vacant(e) => {
                                if let Some(c) = tcp::Connection::accept(
                                    &nic,
                                    ip_hdr,
                                    tcp_hdr,
                                    &buf[data_idx..nbytes],
                                )? {
                                    e.insert(c);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("ignoreing weird tcp packet {:?}", e)
                    }
                }
            }
            Err(e) => {
                eprintln!("ignoreing weird packet {:?}", e)
            }
        }
    }

    // Ok(())
}
