// This file contains all the stuff needed for the "Update Checker" and for the future "Autoupdater".

use restson::RestPath;
use serde_derive::{Serialize, Deserialize};

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::{Write, BufWriter};

use crate::RPFM_PATH;
use crate::error;
use crate::packedfile::db::schemas::Schema;

/// Custom type for the versions of the schemas.
pub type Versions = BTreeMap<String, u32>;

#[derive(Serialize,Deserialize,Debug)]
pub struct LastestRelease {
    pub name: String,
    pub html_url: String,
    pub body: String
}

#[derive(Serialize,Deserialize,Debug,PartialEq)]
pub struct Schemas {
    pub schema_file: String,
    pub version: u32,
}

// Path of the REST endpoint: e.g. http://<baseurl>/anything
impl RestPath<()> for LastestRelease {
    fn get_path(_: ()) -> Result<String, restson::Error> {
        Ok(String::from("repos/frodo45127/rpfm/releases/latest"))
    }
}

/// This function gets the lastest version of the schemas from RPFM's main repo, and updates them if needed.
pub fn update_schemas(
    local_versions: &Versions,
    remote_versions: &Versions,
) -> error::Result<()> {

    // For each schema in the repo, get his equivalent local_schema's path.
    for (remote_schema_name, remote_schema_version) in remote_versions {

        // If the schema exist in our local_versions, depending on the version we update it or not.
        if let Some(local_schema_version) = local_versions.get(remote_schema_name) {

            // If it's an update over our own schema, we download it and overwrite the current schema.
            // NOTE: Github's API has a limit of 1MB per file, so we take it directly from raw.githubusercontent.com instead.
            if remote_schema_version > local_schema_version {
                let response: Schema = reqwest::get(&format!("https://raw.githubusercontent.com/Frodo45127/rpfm/master/schemas/{}", remote_schema_name))?.json()?;
                response.save(remote_schema_name)?;
            }
        }

        // Otherwise, it's a new schema, so we just download it.
        else {
            let response: Schema = reqwest::get(&format!("https://raw.githubusercontent.com/Frodo45127/rpfm/master/schemas/{}", remote_schema_name))?.json()?;
            response.save(remote_schema_name)?;
        }
    }

    // Now we update the "versions.json" to reflect the update.
    let versions_path = RPFM_PATH.to_path_buf().join(PathBuf::from("schemas/versions.json"));
    let mut file = BufWriter::new(File::create(&versions_path)?);
    file.write_all(serde_json::to_string_pretty(&remote_versions)?.as_bytes())?;

    // If we reach this place, return success.
    Ok(())
}
