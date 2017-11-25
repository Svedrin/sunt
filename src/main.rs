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
extern crate uname;

use std::collections::BTreeMap;
use std::net::{UdpSocket,SocketAddr};
use snmp::SnmpPdu;
use snmp::pdu;

mod errors {
    error_chain! { }
}

use errors::*;

mod value;
use value::Value;

mod mib_sys;

fn run(port: u32, community: &str) -> Result<()> {
    let addr: SocketAddr = format!("[::]:{}", port).parse()
        .chain_err(|| "Could not parse address")?;

    let socket = UdpSocket::bind(addr)
        .chain_err(|| "Could not bind to socket")?;

    let mut values: BTreeMap<String, Value> = BTreeMap::new();
    mib_sys::get_system(&mut values, "1.3.6.1.2.1.1");

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

                let mut outbuf = pdu::Buf::default();

                // Convert stuff into the correct types. Actually I'm pretty sure this shouldn't be
                // so complicated...
                // First, convert stringly-typed OIDs to lists-of-numbers,
                // then   convert the lists-of-numbers to refs-to-lists-of-numbers which is what
                //        build_response expects,
                // but do it in such a way that the values the refs point *to* live long enough
                // by keeping the originals in a variable till the end of this scope.

                let mut vals : Vec<(Vec<u32>, snmp::Value)> = values
                    .iter()
                    .take(10)
                    .map(|(oid_str, val)| (
                        oid_str
                            .split(".")
                            .map(|i| i.parse::<u32>().unwrap())
                            .collect::<Vec<u32>>(),
                        val.as_snmp_value()
                    ))
                    .collect();

                vals.push( (vec![0,0], snmp::Value::EndOfMibView) );

                let refd_vals = vals
                    .iter()
                    .map(|&(ref oid, ref val)| (&oid[..], val))
                    .collect::<Vec<(&[u32], &snmp::Value)>>();

                pdu::build_response(
                    &community.as_bytes(),
                    req.req_id,
                    &refd_vals[..],
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
