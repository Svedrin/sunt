use std::fs::File;
use std::io::{Read};
use std::path::PathBuf;

use yaml_rust::{Yaml,YamlLoader};


pub fn load_conf(confpath: PathBuf) -> Option<Yaml> {
    let conffile = &mut String::new();
    File::open(confpath).ok()?
        .read_to_string(conffile).ok()?;

    let mut confs = YamlLoader::load_from_str(conffile.as_str()).ok()?;
    if confs.len() < 1 {
        return None;
    }
    Some(confs.pop().unwrap())
}
