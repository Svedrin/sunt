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
mod mib_disks;

fn run(port: u32, community: &str) -> Result<()> {
    let addr: SocketAddr = format!("[::]:{}", port).parse()
        .chain_err(|| "Could not parse address")?;

    let socket = UdpSocket::bind(addr)
        .chain_err(|| "Could not bind to socket")?;

    let mut values: BTreeMap<String, Value> = BTreeMap::new();
    mib_sys::get_system(&mut values, "1.3.6.1.2.1.1");
    mib_disks::get_disks(&mut values, "1.3.6.1.4.1.2021.13.15.1.1");

    let mut buf = [0 as u8; 16 * 1024];
    loop {
        if let Ok((data_len, client_addr)) = socket.recv_from(&mut buf) {
            let pdu_bytes = &buf[0..data_len];

            println!("got {} dataz from client {}: {:?}", data_len, client_addr, pdu_bytes);

            if let Ok(req) = SnmpPdu::from_bytes(pdu_bytes) {
                println!("request type: {:?}", req.message_type);
                println!("req_id:        {}", req.req_id);

                let mut start_from_oid = None;

                for (name, val) in req.varbinds {
                    start_from_oid = Some(name.to_string());
                    break;
                }

                respond(&values, start_from_oid, req.req_id, client_addr, &community, &socket)?;
            }
        }
    }
}


fn respond (
    values:         &BTreeMap<String, Value>,
    start_from_oid: Option<String>,
    req_id:         i32,
    client_addr:    SocketAddr,
    community:      &str,
    socket:         &UdpSocket
) -> Result<()> {
    let mut outbuf = pdu::Buf::default();

    // Convert stuff into the correct types. Actually I'm pretty sure this shouldn't be
    // so complicated...
    // First, convert stringly-typed OIDs to lists-of-numbers,
    // then   convert the lists-of-numbers to refs-to-lists-of-numbers which is what
    //        build_response expects,
    // but do it in such a way that the values the refs point *to* live long enough
    // by keeping the originals in a variable till the end of this scope.

    // If we don't have a start OID, start right away
    let mut found_start = start_from_oid.is_none();

    let mut vals: Vec<(Vec<u32>, snmp::Value)> = Vec::new();

    for (oid_str, val) in values {
        let oid_vec = oid_str
            .split(".")
            .map(|i| i.parse::<u32>().unwrap())
            .collect::<Vec<u32>>();

        if !found_start && oid_str.starts_with(start_from_oid.as_ref().unwrap()) {
            println!("Found start OID {} ?= {}", start_from_oid.as_ref().unwrap(), oid_str);
            found_start = true;
            continue;
        }

        if found_start {
            vals.push( (oid_vec, val.as_snmp_value()) );
        }

        if vals.len() >= 100 {
            break
        }
    }

    if !vals.is_empty() {
        let mut refd_vals = vals
            .iter()
            .map(|&(ref oid, ref val)| (&oid[..], val))
            .collect::<Vec<(&[u32], &snmp::Value)>>();

        pdu::build_response(
            &community.as_bytes(),
            req_id,
            &refd_vals[..],
            &mut outbuf
        );
    }
    else {
        pdu::build_response(
            &community.as_bytes(),
            req_id,
            &[(&[0, 0], &snmp::Value::EndOfMibView)],
            &mut outbuf
        );
    }

    socket.send_to(&outbuf[..], client_addr)
        .chain_err(|| "Could not send")?;

    Ok(())
}


fn main(){
    if let Err(ref e) = run(1161, "sunt") {
        eprintln!("error: {}", e);
    }
}
