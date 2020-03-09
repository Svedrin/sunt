use std::collections::{BTreeMap};
use value::{Value,str_from_file,u32_from_file};
use oid::OID;
use std::fs::File;
use std::io::{BufReader,BufRead};
use std::path::PathBuf;

#[derive(Debug)]
enum IfaceClass {
    Physical,
    Bonding,
    VLAN,
    Bridge,
    Virtual
}

/**
 * Given a device name such as virbr0, figure out what kind of interface that is.
 */
fn classify_interface(ifname: &String) -> IfaceClass {
    let sys = PathBuf::from("/sys/class/net").join(ifname);
    if sys.join("device").exists() {
        return IfaceClass::Physical;
    }
    if sys.join("bonding").exists() {
        return IfaceClass::Bonding;
    }
    if sys.join("bridge").exists() {
        return IfaceClass::Bridge;
    }
    if sys.join("master").exists() {
        if let Ok(vconfig) = File::open("/proc/net/vlan/config") {
            for vcline in BufReader::new(vconfig).lines().skip(2) {
                if vcline.unwrap().split_whitespace().nth(0).unwrap() == ifname {
                    return IfaceClass::VLAN;
                }
            }
        }
    }
    return IfaceClass::Virtual;
}

pub fn get_interfaces(values: &mut BTreeMap<OID, Value>, if_table_oid: &str, extended_oid: &str) {
    // ifTable

    if let Ok(netdevstats) = File::open("/proc/net/dev") {
        let mut iface_idx = 1;

        for line in BufReader::new(netdevstats).lines().skip(2) {
            let line = line.unwrap();
            let parts = line.split_whitespace().collect::<Vec<&str>>();
            // Parts:
            //    Interface: (yes, there's a colon in there)
            // Rx bytes packets errs drop fifo frame compressed multicast
            // Tx bytes packets errs drop fifo colls carrier compressed

            let ifname = String::from(parts[0].trim_end_matches(":"));
            let ifsys = PathBuf::from("/sys/class/net").join(&ifname);
            let ifclass = classify_interface(&ifname);

            match ifclass {
                IfaceClass::Virtual => continue,
                _ => ()
            }

            // ifTable

            values.insert( // ifIndex
                OID::from_parts_and_instance(&[if_table_oid, "1"], iface_idx),
                Value::Integer(iface_idx as i64)
            );
            values.insert( // ifDescr
                OID::from_parts_and_instance(&[if_table_oid, "2"], iface_idx),
                Value::OctetString(ifname.to_owned())
            );
            values.insert( // ifType
                OID::from_parts_and_instance(&[if_table_oid, "3"], iface_idx),
                Value::Integer(match ifclass {
                    IfaceClass::Physical if ifname.starts_with("wl") => 71,
                    IfaceClass::VLAN     => 135,
                    _                    => 6
                } as i64)
            );
            values.insert( // ifMtu
                OID::from_parts_and_instance(&[if_table_oid, "4"], iface_idx),
                Value::Integer(u32_from_file(&ifsys.join("mtu")).unwrap() as i64)
            );
            values.insert( // ifSpeed
                OID::from_parts_and_instance(&[if_table_oid, "5"], iface_idx),
                Value::Unsigned32(
                    u32_from_file(&ifsys.join("speed")).or(Some(0)).unwrap().saturating_mul(1000000)
                )
            );
            // ifPhysAddress not supported
            // ifAdminStatus not supported
            values.insert( // ifOperStatus
                OID::from_parts_and_instance(&[if_table_oid, "8"], iface_idx),
                Value::Integer(
                    match str_from_file(&ifsys.join("operstate")).unwrap() == "up" {
                        true  => 1,
                        false => 2
                    }
                )
            );
            // ifLastChange not supported
            values.insert( // ifInOctets
                OID::from_parts_and_instance(&[if_table_oid, "10"], iface_idx),
                Value::Counter32(parts[1].parse::<u64>().unwrap())
            );
            values.insert( // ifInUcastPkts
                OID::from_parts_and_instance(&[if_table_oid, "11"], iface_idx),
                Value::Counter32(parts[2].parse::<u64>().unwrap())
            );
            values.insert( // ifInNUcastPkts
                OID::from_parts_and_instance(&[if_table_oid, "12"], iface_idx),
                Value::Counter32(parts[8].parse::<u64>().unwrap())
            );
            values.insert( // ifInDiscards
                OID::from_parts_and_instance(&[if_table_oid, "13"], iface_idx),
                Value::Counter32(parts[4].parse::<u64>().unwrap())
            );
            values.insert( // ifInErrors
                OID::from_parts_and_instance(&[if_table_oid, "14"], iface_idx),
                Value::Counter32(parts[3].parse::<u64>().unwrap())
            );
            // IfInUnknownProtos not supported
            values.insert( // ifOutOctets
                OID::from_parts_and_instance(&[if_table_oid, "16"], iface_idx),
                Value::Counter32(parts[9].parse::<u64>().unwrap())
            );
            values.insert( // ifOutUcastPkts
                OID::from_parts_and_instance(&[if_table_oid, "17"], iface_idx),
                Value::Counter32(parts[10].parse::<u64>().unwrap())
            );
            // ifOutNUcastPkts not supported
            values.insert( // ifOutDiscards
                OID::from_parts_and_instance(&[if_table_oid, "19"], iface_idx),
                Value::Counter32(parts[12].parse::<u64>().unwrap())
            );
            values.insert( // ifOutErrors
                OID::from_parts_and_instance(&[if_table_oid, "20"], iface_idx),
                Value::Counter32(parts[11].parse::<u64>().unwrap())
            );
            // ifOutQLen not supported
            // ifSpecific not supported

            // extended Table (no idea if it has a name)

            values.insert( // ifName
                OID::from_parts_and_instance(&[extended_oid, "1"], iface_idx),
                Value::OctetString(ifname.to_owned())
            );
            values.insert( // ifHCInOctets
                OID::from_parts_and_instance(&[extended_oid, "6"], iface_idx),
                Value::Counter64(parts[1].parse::<u64>().unwrap())
            );
            values.insert( // ifHCInUcastPkts
                OID::from_parts_and_instance(&[extended_oid, "7"], iface_idx),
                Value::Counter64(parts[2].parse::<u64>().unwrap())
            );
            values.insert( // ifHCOutOctets
                OID::from_parts_and_instance(&[extended_oid, "10"], iface_idx),
                Value::Counter64(parts[9].parse::<u64>().unwrap())
            );
            values.insert( // ifHCOutUcastPkts
                OID::from_parts_and_instance(&[extended_oid, "11"], iface_idx),
                Value::Counter64(parts[10].parse::<u64>().unwrap())
            );
            values.insert( // ifHighSpeed
                OID::from_parts_and_instance(&[extended_oid, "15"], iface_idx),
                Value::Unsigned32(
                    u32_from_file(&ifsys.join("speed")).or(Some(0)).unwrap()
                )
            );

            iface_idx += 1;
        }
    }
}


