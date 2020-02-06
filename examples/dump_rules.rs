use netlink_packet_route::{
    constants::*, nlas::rule::Nla, NetlinkHeader, NetlinkMessage, NetlinkPayload, RtnlMessage,
    RuleHeader, RuleMessage,
};
use netlink_sys::{Protocol, Socket, SocketAddr};
use std::net::Ipv4Addr;
use std::string::ToString;

fn main() {
    let mut socket = Socket::new(Protocol::Route).unwrap();
    let _port_number = socket.bind_auto().unwrap().port_number();
    socket.connect(&SocketAddr::new(0, 0)).unwrap();

    let msg = RuleMessage::default();

    let mut packet = NetlinkMessage {
        header: NetlinkHeader {
            flags: NLM_F_REQUEST | NLM_F_DUMP,
            ..Default::default()
        },
        payload: NetlinkPayload::from(RtnlMessage::GetRule(RuleMessage::default())),
    };

    packet.finalize();

    let mut buf = vec![0; packet.header.length as usize];

    // Before calling serialize, it is important to check that the buffer in which we're emitting is big
    // enough for the packet, other `serialize()` panics.

    assert!(buf.len() == packet.buffer_len());

    packet.serialize(&mut buf[..]);

    println!(">>> {:?}", packet);
    if let Err(e) = socket.send(&buf[..], 0) {
        println!("SEND ERROR {}", e);
    }

    let mut receive_buffer = vec![0; 4096];
    let mut offset = 0;

    // we set the NLM_F_DUMP flag so we expect a multipart rx_packet in response.
    while let Ok(size) = socket.recv(&mut receive_buffer[..], 0) {
        loop {
            let bytes = &receive_buffer[offset..];
            let rx_packet = <NetlinkMessage<RtnlMessage>>::deserialize(bytes).unwrap();
            println!("<<< {:?}", rx_packet);

            if rx_packet.payload == NetlinkPayload::Done {
                println!("Done!");
                return;
            }

            offset += rx_packet.header.length as usize;
            if offset == size || rx_packet.header.length == 0 {
                offset = 0;
                break;
            }
        }
    }
}
