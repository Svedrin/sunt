use std::collections::{BTreeMap};
use value::Value;
use oid::OID;
use std::process::Command;
use yaml_rust::Yaml;

pub fn get_extend(values: &mut BTreeMap<OID, Value>, conf: &Option<Yaml>, extend_oid: &str) {
    if conf.is_none() {
        return;
    }
    let conf = conf.as_ref().unwrap();
    for (name, command) in conf["extend"].as_hash().unwrap() {
        let name = name
            .as_str()
            .expect(&format!("Name is not a string: {:?}", name));
        let namepart = OID::asciify_part(&String::from(name));
        let lenpart = format!("{}", name.len());

        let output = Command::new(command["cmd"].as_str().expect("no command given"))
            .args(
                command["args"]
                    .as_vec()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|arg| arg.as_str().expect("arg is not a string"))
                    .collect::<Vec<&str>>()
            )
            .output()
            .expect("Could not execute command");

        let output_string = String::from_utf8(output.stdout).unwrap();
        let output_first = output_string.lines().nth(0).unwrap_or("");
        let output_lines = output_string.lines().count();

        values.insert( // nsExtendOutput1Line = 1
            OID::from_parts(&[extend_oid, "1", &lenpart, &namepart]),
            Value::OctetString(String::from(output_first))
        );
        values.insert( // nsExtendOutputFull = 2
            OID::from_parts(&[extend_oid, "2", &lenpart, &namepart]),
            Value::OctetString(String::from(output_string.trim_end()).to_owned())
        );
        values.insert( // nsExtendOutNumLines = 3
            OID::from_parts(&[extend_oid, "3", &lenpart, &namepart]),
            Value::Integer(output_lines as i64)
        );
        values.insert( // nsExtendResult = 4
            OID::from_parts(&[extend_oid, "4", &lenpart, &namepart]),
            Value::Integer(output.status.code().unwrap() as i64)
        );
    }
}
