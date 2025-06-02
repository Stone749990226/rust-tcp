use std::io;

use etherparse::IpNumber;

fn main() -> io::Result<()> {
    let nic = tun_tap::Iface::new("tun0", tun_tap::Mode::Tun)?;
    let mut buf = [0u8; 1504];
    loop {
        let nbytes = nic.recv(&mut buf[..])?;
        let flags = u16::from_be_bytes([buf[0], buf[1]]);
        let proto = u16::from_be_bytes([buf[2], buf[3]]);
        if proto != 0x0800 {
            // not ipv4
            continue;
        }
        match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..nbytes]) {
            Ok(p) => {
                let src = p.source_addr();
                let dst = p.destination_addr();
                let proto = p.protocol();
                if proto != IpNumber::TCP {
                    continue;
                }
                match etherparse::TcpHeaderSlice::from_slice(&buf[4 + p.slice().len()..]) {
                    Ok(p) => {
                        eprintln!(
                            "{src} -> {dst} {}b of tcp to port {}",
                            p.slice().len(),
                            p.destination_port()
                        );
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

    Ok(())
}
