use snmp;
use std::fs::File;
use std::path::PathBuf;
use std::io::{BufReader,BufRead};

#[allow(dead_code)]
pub enum Value<'a> {
    Boolean(bool),
    Null,
    Integer(i64),
    OctetString(String),
    OctetStr(&'a str),

    IpAddress([u8;4]),
    Counter32(u64),
    Unsigned32(u32),
    Timeticks(u32),
    Counter64(u64),
}

impl<'a> Value<'a> {
    pub fn as_snmp_value(&self) -> snmp::Value {
        match self {
            &Value::Boolean(bool_)          => snmp::Value::Boolean(bool_),
            &Value::Null                    => snmp::Value::Null,
            &Value::Integer(i64_)           => snmp::Value::Integer(i64_),
            &Value::OctetString(ref string) => snmp::Value::OctetString(string.as_bytes()),
            &Value::OctetStr(str_)          => snmp::Value::OctetString(str_.as_bytes()),
            &Value::IpAddress(ip)           => snmp::Value::IpAddress(ip),
            &Value::Counter32(u64_)         => snmp::Value::Counter32((u64_ & 0xFFFFFFFF) as u32),
            &Value::Unsigned32(u32_)        => snmp::Value::Unsigned32(u32_),
            &Value::Timeticks(u32_)         => snmp::Value::Timeticks(u32_),
            &Value::Counter64(u64_)         => snmp::Value::Counter64(u64_),
        }
    }
}

pub fn str_from_file(fpath: &PathBuf) -> Option<String> {
    BufReader::new(File::open(fpath).unwrap())
        .lines()
        .nth(0)?
        .ok()
}

pub fn u32_from_file(fpath: &PathBuf) -> Option<u32> {
    str_from_file(fpath)?
        .split(".")
        .next()?
        .parse::<u32>()
        .ok()
}
