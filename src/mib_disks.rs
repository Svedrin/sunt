use std::collections::{BTreeMap,HashSet};
use value::Value;
use oid::OID;
use std::fs;
use std::fs::File;
use std::mem;
use std::io::{BufReader,BufRead,Error};
use std::path::PathBuf;
use std::ffi::CString;
use libc;


/**
 * device is some path under /dev. Resolve symlinks down to the actual /dev/something.
 */
fn resolve_dev_symlinks(input: PathBuf) -> PathBuf {
    if let Ok(target) = input.read_link() {
        resolve_dev_symlinks(
            input.parent()
                .unwrap()
                .join(target)
                .canonicalize()
                .unwrap()
        )
    }
    else {
        // Probably not a symlink
        input
    }
}

/**
 * device is a /dev/dm-*. Search /dev/mapper for a more meaningful name.
 */
fn canonicalize_dm_name(devpath: PathBuf) -> Option<String> {
    if let Ok(entries) = fs::read_dir("/dev/mapper") {
        for entry in entries {
            if let Ok(entry) = entry {
                // Resolve symlink
                if let Ok(alias_path) = fs::read_link(entry.path()) {
                    // Turn symlink "../dm-X" into "/dev/dm-X"
                    let mut base = PathBuf::from("/dev/mapper");
                    base.push(&alias_path);
                    if fs::canonicalize(base).unwrap() == devpath {
                        // Found our /dev/dm-X! See if the /dev/mapper name has a - in it (LV)
                        let file_name_string = entry.file_name()
                            .into_string()
                            .unwrap();
                        if file_name_string.contains("-") {
                            // This is probably an LV
                            let parts = file_name_string
                                .splitn(2, "-")
                                .map(|part| part.replace("--", "-"))
                                .collect::<Vec<String>>();
                            let lvpath = format!("{}/{}", parts[0], parts[1]);
                            // Check if /dev/vg/lv exists
                            if PathBuf::from("/dev").join(&lvpath).exists() {
                                return Some(lvpath);
                            }
                        }
                        // Something else, return as-is.
                        return Some(file_name_string);
                    }
                }
            }
        }
    }
    None
}

pub fn get_filesystems(
    values: &mut BTreeMap<OID, Value>,
    hr_storage_table_oid: &str,
    dsk_table_oid: &str
) {
    if let Ok(diskstats) = File::open("/proc/mounts") {
        let mut disk_idx = 1;
        let dups : &mut HashSet<u64> = &mut HashSet::new();

        for line in BufReader::new(diskstats).lines() {
            let line = line.unwrap();
            let parts = line.split_whitespace().collect::<Vec<&str>>();
            // Parts:
            // device mountpoint fstype options dump pass

            let device = String::from(parts[0]);
            let devpath = resolve_dev_symlinks(PathBuf::from(&device));
            let mountpoint = String::from(parts[1]);

            if !device.starts_with("/dev") {
                continue;
            }

            let fsstat = unsafe {
                let mut fsstat: libc::statvfs64 = mem::zeroed();
                let path = CString::new(&mountpoint[..]).unwrap();
                if libc::statvfs64(path.as_ptr(), &mut fsstat) != 0 {
                    Err(Error::last_os_error())
                }
                else {
                    Ok(fsstat)
                }
            };

            if fsstat.is_err() {
                // TODO: We should probably log this or sumt'n
                continue;
            }

            let fsstat = fsstat.unwrap();

            let alias =
                if devpath.to_str().unwrap().starts_with("/dev/dm-") {
                    // Find a name better suited for dem humans
                    canonicalize_dm_name(devpath)
                        .and_then(|name| Some(format!("/dev/{}", name)))
                }
                else {
                    None
                };


            // Filter dups (bind mounts, e.g. docker)
            if dups.contains(&fsstat.f_fsid) {
                continue;
            }
            else {
                dups.insert(fsstat.f_fsid);
            }

            // hrStorageTable

            values.insert( // hrStorageIndex
                OID::from_parts_and_instance(&[hr_storage_table_oid, "1"], disk_idx),
                Value::Integer(disk_idx as i64)
            );
            values.insert( // hrStorageType
                OID::from_parts_and_instance(&[hr_storage_table_oid, "2"], disk_idx),
                Value::Null
            );
            values.insert( // hrStorageDescr
                OID::from_parts_and_instance(&[hr_storage_table_oid, "3"], disk_idx),
                Value::OctetString(mountpoint.to_owned())
            );
            values.insert( // hrStorageAllocationUnits
                OID::from_parts_and_instance(&[hr_storage_table_oid, "4"], disk_idx),
                Value::Integer(fsstat.f_frsize as i64)
            );
            values.insert( // hrStorageSize
                OID::from_parts_and_instance(&[hr_storage_table_oid, "5"], disk_idx),
                Value::Integer(fsstat.f_blocks as i64)
            );
            values.insert( // hrStorageUsed
                OID::from_parts_and_instance(&[hr_storage_table_oid, "6"], disk_idx),
                Value::Integer((fsstat.f_blocks - fsstat.f_bfree) as i64)
            );
            // hrStorageAllocationFailures is unsupported

            // dskTable
            //  dskIndex  dskPath dskDevice dskMinimum dskMinPercent  dskTotal dskAvail  dskUsed
            //  dskPercent dskPercentNode dskTotalLow dskTotalHigh dskAvailLow dskAvailHigh
            //  dskUsedLow dskUsedHigh dskErrorFlag                           dskErrorMsg

            values.insert( // dskIndex
                OID::from_parts_and_instance(&[dsk_table_oid, "1"], disk_idx),
                Value::Integer(disk_idx as i64)
            );
            values.insert( // dskPath
                OID::from_parts_and_instance(&[dsk_table_oid, "2"], disk_idx),
                Value::OctetString(mountpoint)
            );
            values.insert( // dskDevice
                OID::from_parts_and_instance(&[dsk_table_oid, "3"], disk_idx),
                Value::OctetString(alias.or(Some(device)).unwrap())
            );
            values.insert( // dskMinimum
                OID::from_parts_and_instance(&[dsk_table_oid, "4"], disk_idx),
                Value::Integer(0)
            );
            values.insert( // dskMinPercent
                OID::from_parts_and_instance(&[dsk_table_oid, "5"], disk_idx),
                Value::Integer(-1)
            );
            values.insert( // dskTotal
                OID::from_parts_and_instance(&[dsk_table_oid, "6"], disk_idx),
                Value::Integer( (fsstat.f_blocks * fsstat.f_frsize / 1024) as i64 )
            );
            values.insert( // dskAvail
                OID::from_parts_and_instance(&[dsk_table_oid, "7"], disk_idx),
                Value::Integer( (fsstat.f_bavail * fsstat.f_frsize / 1024) as i64 )
            );

            let f_bused = fsstat.f_blocks - fsstat.f_bfree;

            values.insert( // dskUsed
                OID::from_parts_and_instance(&[dsk_table_oid, "8"], disk_idx),
                Value::Integer( (f_bused * fsstat.f_frsize / 1024) as i64 )
            );
            values.insert( // dskPercent
                OID::from_parts_and_instance(&[dsk_table_oid, "9"], disk_idx),
                Value::Integer( (f_bused * 100 / fsstat.f_blocks) as i64 )
            );

            if fsstat.f_files != 0 {
                let f_fused = fsstat.f_files - fsstat.f_ffree;

                values.insert(// dskPercentNode
                    OID::from_parts_and_instance(&[dsk_table_oid, "10"], disk_idx),
                    Value::Integer((f_fused * 100 / fsstat.f_files) as i64)
                );
            }

            // Rest: Unsupported

            disk_idx += 1;
        }
    }
}

pub fn get_disks(values: &mut BTreeMap<OID, Value>, base_oid: &str) {
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
                alias = canonicalize_dm_name(devpath);
            }

            let reads  = parts[3].parse::<u32>().unwrap();
            let writes = parts[4].parse::<u32>().unwrap();
            let read_bytes = parts[5].parse::<u64>().unwrap() * 512;
            let wrtn_bytes = parts[6].parse::<u64>().unwrap() * 512;

            values.insert( // diskIOIndex
                OID::from_parts_and_instance(&[base_oid,  "1"], disk_idx),
                Value::Integer(disk_idx as i64)
            );
            values.insert( // diskIODevice
                OID::from_parts_and_instance(&[base_oid,  "2"], disk_idx),
                Value::OctetString(alias.or(Some(device)).unwrap())
            );
            // NRead, NWritten (old sucky 32 bit counters). I hope these conversions are correct :/
            values.insert( // diskIONRead
                OID::from_parts_and_instance(&[base_oid,  "3"], disk_idx),
                Value::Counter32((read_bytes & 0xFFFFFFFF) as u32)
            );
            values.insert( // diskIONWritten
                OID::from_parts_and_instance(&[base_oid,  "4"], disk_idx),
                Value::Counter32((wrtn_bytes & 0xFFFFFFFF) as u32)
            );
            // reads, writes
            values.insert( // diskIOReads
                OID::from_parts_and_instance(&[base_oid,  "5"], disk_idx),
                Value::Counter32(reads)
            );
            values.insert( // diskIOWrites
                OID::from_parts_and_instance(&[base_oid,  "6"], disk_idx),
                Value::Counter32(writes)
            );
            // 7, 8: ???
            // diskIOLA1, 5, 15
            values.insert( // diskIOLA1
                OID::from_parts_and_instance(&[base_oid,  "9"], disk_idx),
                Value::Integer(0)
            );
            values.insert( // diskIOLA5
                OID::from_parts_and_instance(&[base_oid, "10"], disk_idx),
                Value::Integer(0)
            );
            values.insert( // diskIOLA15
                OID::from_parts_and_instance(&[base_oid, "11"], disk_idx),
                Value::Integer(0)
            );
            // NReadX, NWrittenX (new shiny 64 bit counters)
            values.insert( // diskIONReadX
                OID::from_parts_and_instance(&[base_oid, "12"], disk_idx),
                Value::Counter64(read_bytes)
            );
            values.insert( // diskIONWrittenX
                OID::from_parts_and_instance(&[base_oid, "13"], disk_idx),
                Value::Counter64(wrtn_bytes)
            );

            disk_idx += 1;
        }
    }
}
