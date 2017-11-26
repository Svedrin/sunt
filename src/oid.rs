use snmp;
use std::cmp::Ordering;
use std::fmt;

pub struct OID {
    oid_str: String,
    oid_vec: Vec<u32>,
}

impl OID {
    pub fn from_string(input: String) -> OID {
        let oid_str = input.to_owned();
        let oid_vec = oid_str
            .split(".")
            .map(|i| i.parse::<u32>().unwrap())
            .collect::<Vec<u32>>();
        OID { oid_str: oid_str, oid_vec: oid_vec }
    }

    pub fn from_vec(input: &Vec<u32>) -> OID {
        let oid_str = input
            .iter()
            .map(|i| format!("{}", i))
            .collect::<Vec<String>>()
            .join(".");
        let oid_vec = input.to_owned();
        OID { oid_str: oid_str, oid_vec: oid_vec }
    }

    pub fn from_object_identifier(input : snmp::ObjectIdentifier) -> OID {
        OID::from_string(input.to_string())
    }

    pub fn from_parts(input: &[&str]) -> OID {
        OID::from_string(input.join("."))
    }

    pub fn from_parts_and_instance(input: &[&str], instance: u32) -> OID {
        OID::from_string(format!("{}.{}", input.join("."), instance))
    }

    pub fn as_vec(&self) -> &Vec<u32> {
        &self.oid_vec
    }

    pub fn as_string(&self) -> &String {
        &self.oid_str
    }

    pub fn is_subtree_of(&self, subtree: &OID) -> bool {
        return self.oid_str.starts_with(subtree.str());
    }

    pub fn str(&self) -> &str {
        return &self.oid_str;
    }
}

impl PartialEq for OID {
    fn eq(&self, other: &OID) -> bool {
        self.as_vec() == other.as_vec()
    }
}

impl Eq for OID {}

impl Ord for OID {
    fn cmp(&self, other: &OID) -> Ordering {
        for (mine, theirs) in self.as_vec().iter().zip(other.as_vec().iter()) {
            if mine != theirs {
                return mine.cmp(theirs);
            }
        }
        return Ordering::Equal;
    }
}

impl PartialOrd for OID {
    fn partial_cmp(&self, other: &OID) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for OID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.oid_str)
    }
}
