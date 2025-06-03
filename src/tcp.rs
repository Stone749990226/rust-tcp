use etherparse::{IpNumber, Ipv4Header, Ipv4HeaderSlice, TcpHeader, TcpHeaderSlice};
use std::io::Write;
use tun_tap::Iface;
pub enum State {
    Closed,
    Listen,
    SynRcvd,
}

pub struct Connection {
    state: State,
    recv: RecvSequenceSpace,
    send: SendSequenceSpace,
}

/// Send Sequence Variables
///   SND.UNA - send unacknowledged
///   SND.NXT - send next
///   SND.WND - send window
///   SND.UP  - send urgent pointer
///   SND.WL1 - segment sequence number used for last window update
///   SND.WL2 - segment acknowledgment number used for last window
///             update
///   ISS     - initial send sequence number
/// ```md
/// Send Sequence Space
///      1         2          3          4
/// ----------|----------|----------|----------
///        SND.UNA    SND.NXT    SND.UNA
///                             +SND.WND
/// 1 - 已发送并且已经收到确认了
/// 2 - 已发送但是还没有收到确认
/// 3 - 可以发送但是现在还没有发
/// 4 - 未来要发送但是现在不能发
/// ```

struct SendSequenceSpace {
    /// send unacknowledged
    una: u32,
    /// send next
    nxt: u32,
    /// send window
    wnd: u16,
    /// send urgent pointer
    up: bool,
    /// segment sequence number used for last window update
    wl1: usize,
    /// segment acknowledgment number used for last window update
    wl2: usize,
    /// initial send sequence number
    iss: u32,
}

/// Receive Sequence Variables
///   RCV.NXT - receive next
///   RCV.WND - receive window
///   RCV.UP  - receive urgent pointer
///   IRS     - initial receive sequence number
///
/// ```md
/// Receive Sequence Space
///     1          2          3      
/// ----------|----------|----------
///        RCV.NXT    RCV.NXT        
///                  +RCV.WND        
///
/// 1 - 已经收到的序列号
/// 2 - 新接受可以使用的序列号      
/// 3 - 未来才能用的序列号
/// ```
struct RecvSequenceSpace {
    /// receive next
    nxt: u32,
    /// receive window
    wnd: u16,
    /// receive urgent pointer
    up: bool,
    /// initial receive sequence number
    irs: u32,
}

impl Connection {
    pub fn accept<'a>(
        nic: &Iface,
        ip_hdr: Ipv4HeaderSlice,
        tcp_hdr: TcpHeaderSlice,
        data: &'a [u8],
    ) -> std::io::Result<Option<Self>> {
        let mut buf = [0u8; 1500];

        if !tcp_hdr.syn() {
            return Ok(None);
        }
        let iss = 0;
        let c = Connection {
            state: State::SynRcvd,
            send: SendSequenceSpace {
                iss,
                una: iss,
                nxt: iss + 1,
                wnd: 10,
                up: false,
                wl1: 0,
                wl2: 0,
            },
            recv: RecvSequenceSpace {
                irs: tcp_hdr.sequence_number(),
                nxt: tcp_hdr.sequence_number() + 1,
                wnd: tcp_hdr.window_size(),
                up: false,
            },
        };

        // 发送syn和ack
        let mut syn_ack = TcpHeader::new(
            tcp_hdr.destination_port(),
            tcp_hdr.source_port(),
            c.send.iss,
            c.send.wnd,
        );
        syn_ack.acknowledgment_number = c.recv.nxt;
        syn_ack.syn = true;
        syn_ack.ack = true;

        let mut ip = Ipv4Header::new(
            syn_ack.header_len() as u16,
            64,
            IpNumber::TCP,
            ip_hdr.destination(),
            ip_hdr.source(),
        )
        .unwrap();

        syn_ack.checksum = syn_ack
            .calc_checksum_ipv4(&ip, &[])
            .expect("calc checksum failed");
        let unwritten = {
            let mut unwritten = &mut buf[..];
            ip.write(&mut unwritten)?;
            syn_ack.write(&mut unwritten)?;
            unwritten.len()
        };
        eprintln!("recv iph  {:02x?}", ip_hdr.slice());
        print_iph(ip_hdr);
        println!();
        eprintln!("recv tcph {:02x?}", tcp_hdr.slice());
        print_tcph(tcp_hdr);
        println!();
        eprintln!("will send {:02x?}", &buf[..buf.len() - unwritten]);

        nic.send(&buf[..buf.len() - unwritten])?;
        Ok(Some(c))
    }

    pub fn on_packet<'a>(
        &mut self,
        nic: &Iface,
        ip_hdr: Ipv4HeaderSlice,
        tcp_hdr: TcpHeaderSlice,
        data: &'a [u8],
    ) -> std::io::Result<()> {
        Ok(())
    }
}

fn print_iph(ip_hdr: Ipv4HeaderSlice) {
    println!("{:?}", ip_hdr.to_header());
}

fn print_tcph(tcp_hdr: TcpHeaderSlice) {
    println!("{:?}", tcp_hdr.to_header());
}
