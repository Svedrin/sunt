use std::collections::BTreeMap;
use value::Value;
use std::fs;
use std::fs::File;
use std::io::{BufReader,BufRead};
use std::path::PathBuf;

pub fn get_filesystems(values: &mut BTreeMap<String, Value>, base_oid: &str) {
    // hrStorageTable -- or
    // UCD-SNMP-MIB::dskTable? (or both?)
    // both, we'll need to gather the dataz anyway, so we might as well encode it into two tables

}

pub fn get_disks(values: &mut BTreeMap<String, Value>, base_oid: &str) {
    // UCD-DISKIO-MIB::diskIOTable
    // diskIOIndex diskIODevice diskIONRead diskIONWritten diskIOReads diskIOWrites ...
    // ... diskIOLA1 diskIOLA5 diskIOLA15 diskIONReadX diskIONWrittenX

    if let Ok(diskstats) = File::open("/proc/diskstats") {
        let mut disk_idx = 1;

        for line in BufReader::new(diskstats).lines() {
            let line = line.unwrap();
            let parts = line.split_whitespace().collect::<Vec<&str>>();
            // Parts:
            // "major", "minor", "device",
            // "rd_ios", "rd_merges", "rd_sectors", "rd_ticks",
            // "wr_ios", "wr_merges", "wr_sectors", "wr_ticks",
            // "ios_in_prog", "tot_ticks", "rq_ticks"

            let device = String::from(parts[2]);
            let devpath = PathBuf::from(format!("/dev/{}", device));
            let mut alias = None;

            if device.starts_with("loop") {
                continue;
            }

            if device.starts_with("dm-") {
                // Find a name better suited for dem humans
                if let Ok(entries) = fs::read_dir("/dev/mapper") {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            if let Ok(alias_path) = fs::read_link(entry.path()) {
                                let mut base = PathBuf::from("/dev/mapper");
                                base.push(&alias_path);
                                if fs::canonicalize(base).unwrap() == devpath {
                                    // Probably this entry points to an LV. Reformat to vg/lv
                                    let parts = entry.file_name()
                                        .into_string()
                                        .unwrap()
                                        .splitn(2, "-")
                                        .map(|part| part.replace("--", "-"))
                                        .collect::<Vec<String>>();
                                    alias = Some(format!("{}/{}", parts[0], parts[1]));
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            if let Some(ref alias) = alias {
                println!("Found an alias for {}: {:?}", device, alias);
            }
            else {
                alias = Some(device);
            }

            let reads  = parts[3].parse::<u32>().unwrap();
            let writes = parts[4].parse::<u32>().unwrap();
            let read_bytes = parts[5].parse::<u64>().unwrap() * 512;
            let wrtn_bytes = parts[6].parse::<u64>().unwrap() * 512;

            values.insert(format!("{}.01.{}", base_oid, disk_idx), Value::Integer(disk_idx));
            values.insert(format!("{}.02.{}", base_oid, disk_idx), Value::OctetString(alias.unwrap()));
            // NRead, NWritten (old sucky 32 bit counters)
            values.insert(format!("{}.03.{}", base_oid, disk_idx), Value::Counter32(0));
            values.insert(format!("{}.04.{}", base_oid, disk_idx), Value::Counter32(0));
            // reads, writes
            values.insert(format!("{}.05.{}", base_oid, disk_idx), Value::Counter32(reads));
            values.insert(format!("{}.06.{}", base_oid, disk_idx), Value::Counter32(writes));
            // 7, 8: ???
            // diskIOLA1, 5, 15
            values.insert(format!("{}.09.{}", base_oid, disk_idx), Value::Integer(0));
            values.insert(format!("{}.10.{}", base_oid, disk_idx), Value::Integer(0));
            values.insert(format!("{}.11.{}", base_oid, disk_idx), Value::Integer(0));
            // NReadX, NWrittenX (new shiny 64 bit counters)
            values.insert(format!("{}.12.{}", base_oid, disk_idx), Value::Counter64(read_bytes));
            values.insert(format!("{}.13.{}", base_oid, disk_idx), Value::Counter64(wrtn_bytes));

            disk_idx += 1;
        }
    }

}
