use std::collections::BTreeMap;
use value::Value;
use oid::OID;
use std::fs;

pub fn get_processes(
    values: &mut BTreeMap<OID, Value>,
    hr_sw_run_table_oid: &str
) {

    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(pid) = entry.file_name().into_string().unwrap().parse::<u32>() {
                    if let Ok(cmdline) = entry.path().join("exe").read_link() {
                        values.insert(
                            OID::from_parts_and_instance(&[hr_sw_run_table_oid, "1"], pid),
                            Value::Integer( pid as i64 )
                        );
                        values.insert(
                            OID::from_parts_and_instance(&[hr_sw_run_table_oid, "2"], pid as u32),
                            Value::OctetString(String::from(cmdline.file_name().unwrap().to_str().unwrap()))
                        );
                        values.insert(
                            OID::from_parts_and_instance(&[hr_sw_run_table_oid, "4"], pid as u32),
                            Value::OctetString(String::from(cmdline.to_str().unwrap()))
                        );
                    }
                }
            }
        }
    }
}
