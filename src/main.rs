// Copyright 2016 Michael Ziegler <diese-addy@funzt-halt.net>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate snmp;
#[macro_use]
extern crate error_chain;

use std::net::{UdpSocket,SocketAddr};
use snmp::SnmpPdu;
use snmp::pdu;
use snmp::Value;

mod errors {
    error_chain! { }
}

use errors::*;

fn run(port: u32, community: &str) -> Result<()> {
    let addr: SocketAddr = format!("[::]:{}", port).parse()
        .chain_err(|| "Could not parse address")?;

    let socket = UdpSocket::bind(addr)
        .chain_err(|| "Could not bind to socket")?;




    let mut buf = [0 as u8; 16 * 1024];
    loop {
        if let Ok((data_len, client_addr)) = socket.recv_from(&mut buf) {
            let pdu_bytes = &buf[0..data_len];

            println!("got {} dataz from client {}: {:?}", data_len, client_addr, pdu_bytes);

            if let Ok(req) = SnmpPdu::from_bytes(pdu_bytes) {
                println!("response type: {:?}", req.message_type);
                println!("req_id:        {}", req.req_id);

                for (name, val) in req.varbinds {
                    println!("{} => {:?}", name, val);
                }

                let syscontact_oid  = &[1,3,6,1,2,1,1,4,0];
                let contact         = Value::OctetString(b"hardcoded string");
                let mut outbuf = pdu::Buf::default();
                pdu::build_response(
                    &community.as_bytes(),
                    req.req_id,
                    &[(syscontact_oid, contact)],
                    &mut outbuf
                );
                socket.send_to(&outbuf[..], client_addr)
                    .chain_err(|| "Could not send")?;
            }
        }
    }
}


fn main(){
    if let Err(ref e) = run(1161, "sunt") {
        eprintln!("error: {}", e);
    }
}
