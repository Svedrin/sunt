use std::collections::BTreeMap;
use std::path::PathBuf;
use uname;
use value::{Value,u32_from_file};
use oid::OID;

pub fn get_system(values: &mut BTreeMap<OID, Value>, base_oid: &str) {
    if let Ok(info) = uname::uname() {
        values.insert(
            OID::from_parts(&[base_oid, "1.0"]),
            Value::OctetString(format!(
                "{} {} {} {} {}",
                info.sysname,
                info.nodename,
                info.release,
                info.version,
                info.machine
            ))
        );

        values.insert(
            OID::from_parts(&[base_oid, "5.0"]),
            Value::OctetString(info.nodename)
        );
    }

    values.insert(OID::from_parts(&[base_oid, "4.0"]), Value::OctetStr("sunt v0.0.1"));
    values.insert(OID::from_parts(&[base_oid, "6.0"]), Value::OctetStr("the cloud, probably"));

    let uptime = u32_from_file(&PathBuf::from("/proc/uptime")).unwrap();
    values.insert(OID::from_parts(&[base_oid, "3.0"]), Value::Timeticks(uptime));
}
