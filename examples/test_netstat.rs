use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};

fn main() {
    let af_flags = AddressFamilyFlags::IPV4;
    let proto_flags = ProtocolFlags::TCP;
    
    match get_sockets_info(af_flags, proto_flags) {
        Ok(sockets) => {
            println!("Found {} TCP connections:", sockets.len());
            for (i, si) in sockets.iter().take(10).enumerate() {
                if let ProtocolSocketInfo::Tcp(tcp) = &si.protocol_socket_info {
                    println!("{}: {}:{} -> {}:{} [{:?}]", 
                        i + 1,
                        tcp.local_addr, tcp.local_port,
                        tcp.remote_addr, tcp.remote_port,
                        tcp.state
                    );
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
