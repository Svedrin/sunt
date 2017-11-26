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
extern crate clap;
extern crate uname;
extern crate libc;
extern crate yaml_rust;

use std::collections::BTreeMap;
use std::net::{UdpSocket,SocketAddr};
use std::time::{Instant,Duration};
use std::path::PathBuf;
use clap::{Arg, App};
use snmp::SnmpPdu;
use snmp::pdu;

mod errors {
    error_chain! { }
}

use errors::*;

mod config;

mod oid;
use oid::OID;

mod value;
use value::Value;

mod mib_sys;
mod mib_disks;
mod mib_net;
mod mib_extend;


fn run(matches: clap::ArgMatches) -> Result<()> {
    let port = matches.value_of("port").unwrap_or("161").parse::<u16>()
        .chain_err(|| "Port argument must be a number between 1 and 65535")?;
    let community = matches.value_of("community").unwrap_or("sunt");

    let mut conf = None;
    if let Some(confpath) = matches.value_of("extend") {
        conf = Some(
            config::load_conf(PathBuf::from(confpath))
                .expect("failed to load config")
        );
    }

    let addr: SocketAddr = format!("[::]:{}", port).parse()
        .chain_err(|| "Could not parse address")?;

    let socket = UdpSocket::bind(addr)
        .chain_err(|| "Could not bind to socket")?;

    socket.set_read_timeout(Some(Duration::new(1, 0)))
        .chain_err(|| "could not set timeout")?;

    let mut values: BTreeMap<OID, Value> = BTreeMap::new();
    let mut last_refresh : Option<Instant> = None;

    let mut buf = [0 as u8; 16 * 1024];
    loop {
        if last_refresh.is_none() ||
           last_refresh.unwrap() + Duration::new(15, 0) < Instant::now() {
            mib_sys::get_system(
                &mut values,
                "1.3.6.1.2.1.1"
            );
            mib_disks::get_disks(
                &mut values,
                "1.3.6.1.4.1.2021.13.15.1.1"
            );
            mib_disks::get_filesystems(
                &mut values,
                "1.3.6.1.2.1.25.2.3.1",
                "1.3.6.1.4.1.2021.9.1"
            );
            mib_net::get_interfaces(
                &mut values,
                "1.3.6.1.2.1.2.2.1",
                "1.3.6.1.2.1.31.1.1.1"
            );
            mib_extend::get_extend(
                &mut values,
                &conf,
                "1.3.6.1.4.1.8072.1.3.2.3.1"
            );
            last_refresh = Some(Instant::now());
        }

        if let Ok((data_len, client_addr)) = socket.recv_from(&mut buf) {
            let pdu_bytes = &buf[0..data_len];

            if let Ok(req) = SnmpPdu::from_bytes(pdu_bytes) {
                let mut start_from_oid = OID::from_parts(&["1"]);

                for (name, _) in req.varbinds {
                    if name.to_string() != "Invalid OID: AsnInvalidLen" {
                        start_from_oid = OID::from_object_identifier(name);
                    }

                    // snmptable likes to query OIDs that don't exist as start OID.
                    // We catch that by checking if the OID actually exists, and if not,
                    // brute-forcing our way up the tree until we find one that does.
                    let mut start_oid_vec = start_from_oid.as_vec().to_owned();
                    while !values.contains_key(&start_from_oid) {
                        start_oid_vec.pop();
                        start_from_oid = OID::from_vec(&start_oid_vec);
                    }
                    break;
                }

                respond(&values, start_from_oid, req.req_id, client_addr, &community, &socket)?;
            }
        }
    }
}


fn respond (
    values:         &BTreeMap<OID, Value>,
    start_from_oid: OID,
    req_id:         i32,
    client_addr:    SocketAddr,
    community:      &str,
    socket:         &UdpSocket
) -> Result<()> {

    let mut outbuf = pdu::Buf::default();
    let mut found_start = false;
    let mut vals: Vec<(&[u32], snmp::Value)> = Vec::new();

    for (oid, val) in values {
        if !found_start && oid.is_subtree_of(&start_from_oid) {
            found_start = true;
            if *oid == start_from_oid {
                continue;
            }
        }

        if found_start {
            vals.push( (&oid.as_vec()[..], val.as_snmp_value()) );
        }

        if vals.len() >= 100 {
            break
        }
    }

    if !vals.is_empty() {
        pdu::build_response(
            &community.as_bytes(),
            req_id,
            &vals[..],
            &mut outbuf
        );
    }
    else {
        pdu::build_response(
            &community.as_bytes(),
            req_id,
            &[(&[0, 0], snmp::Value::EndOfMibView)],
            &mut outbuf
        );
    }

    socket.send_to(&outbuf[..], client_addr)
        .chain_err(|| "Could not send")?;

    Ok(())
}


fn main(){
    let matches = App::new("sunt")
        .version("0.0.1")
        .author("Michael Ziegler <diese-addy@funzt-halt.net>")
        .about("SNMP Agent for Linux")
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .takes_value(true)
            .help("Port number to use [161]"))
        .arg(Arg::with_name("community")
            .short("c")
            .long("community")
            .takes_value(true)
            .help("Community to use in responses [sunt]"))
        .arg(Arg::with_name("extend")
            .short("e")
            .long("extend")
            .takes_value(true)
            .help("Parse config.yaml that defines extends"))
        .get_matches();

    if let Err(ref e) = run(matches) {
        eprintln!("error: {}", e);
    }
}
