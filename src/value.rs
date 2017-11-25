use snmp;
use std::fs::File;
use std::io::{BufReader,BufRead};

#[allow(dead_code)]
pub enum Value {
    Boolean(bool),
    Null,
    Integer(i64),
    OctetString(String),
    OctetStr(&'static str),

    IpAddress([u8;4]),
    Counter32(u32),
    Unsigned32(u32),
    Timeticks(u32),
    Counter64(u64),
}

impl Value {
    pub fn as_snmp_value(&self) -> snmp::Value {
        match self {
            &Value::Boolean(bool_)          => snmp::Value::Boolean(bool_),
            &Value::Null                    => snmp::Value::Null,
            &Value::Integer(i64_)           => snmp::Value::Integer(i64_),
            &Value::OctetString(ref string) => snmp::Value::OctetString(string.as_bytes()),
            &Value::OctetStr(str_)          => snmp::Value::OctetString(str_.as_bytes()),
            &Value::IpAddress(ip)           => snmp::Value::IpAddress(ip),
            &Value::Counter32(u32_)         => snmp::Value::Counter32(u32_),
            &Value::Unsigned32(u32_)        => snmp::Value::Unsigned32(u32_),
            &Value::Timeticks(u32_)         => snmp::Value::Timeticks(u32_),
            &Value::Counter64(u64_)         => snmp::Value::Counter64(u64_),
        }
    }
}

pub fn u32_from_file(fpath: &str) -> u32 {
    String::from_utf8(
    BufReader::new(File::open(fpath).unwrap())
            .split(b'.')
            .next()
            .unwrap()
            .unwrap()
        )
        .unwrap()
        .parse::<u32>()
        .unwrap()
}
