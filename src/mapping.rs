use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mapping {
    pub buttons: Vec<Feature>,
    pub faders: Vec<Feature>,
    pub encoders: Vec<Feature>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    pub name: String,
    pub channel: u8,
    pub message: u8,
}

impl Mapping {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Mapping, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let map = serde_json::from_reader(reader)?;

        Ok(map)
    }
}
