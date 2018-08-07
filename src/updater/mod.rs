// This file contains all the stuff needed for the "Update Checker" and for the future "Autoupdater".
extern crate restson;
extern crate serde_json;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;

use std::path::PathBuf;
use std::fs::File;
use std::io::{Write, BufWriter};
use self::futures::{Future, Stream};
use self::hyper::Client;
use self::tokio_core::reactor::Core;

use error;
use self::restson::RestPath;

#[derive(Serialize,Deserialize,Debug)]
pub struct LastestRelease {
    pub name: String,
    pub html_url: String,
    pub body: String
}

#[derive(Serialize,Deserialize,Debug,PartialEq)]
pub struct Versions {
    pub schemas: Vec<Schemas>,
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

// Path of the REST endpoint: e.g. http://<baseurl>/anything
impl RestPath<()> for Versions {
    fn get_path(_: ()) -> Result<String, restson::Error> {
        Ok(String::from("Frodo45127/rpfm/master/schemas/versions.json"))
    }
}

/// This function gets the lastest version of the schemas from RPFM's main repo, and updates them if needed.
pub fn update_schemas(
    local_versions: Versions,
    current_versions: Versions,
    rpfm_path: &PathBuf
) -> error::Result<()> {

    // For each schema in the repo...
    for (index, schema) in current_versions.schemas.iter().enumerate() {

        // Get the local_schema's path.
        let local_schema_path = rpfm_path.to_path_buf().join(PathBuf::from(format!("schemas/{}", schema.schema_file)));

        // If the schema exist in our local_versions...
        if let Some(local_schema) = local_versions.schemas.get(index) {

            // If the current_version is greater than the local one...
            if schema.version > local_schema.version {

                // Get the file to write the schema on, in case we need it.
                let mut file = BufWriter::new(File::create(&local_schema_path)?);

                // Use Hyper's black magic to download the new schema.
                // NOTE: Github's API has a limit of 1MB per file, so we take it directly from raw.githubusercontent.com instead.
                let mut core = Core::new()?;
                let handle = core.handle();
                let client = Client::configure()
                    .connector(hyper_tls::HttpsConnector::new(4, &handle)?)
                    .build(&handle);

                let uri = format!("https://raw.githubusercontent.com/Frodo45127/rpfm/master/schemas/{}", schema.schema_file).parse().unwrap();
                let work = client.get(uri).and_then(|res| {

                    // Write all to our schema file.
                    res.body().for_each(|chunk| {
                        file.write_all(&chunk).map_err(From::from)
                    })
                });
                core.run(work)?;
            }
        }

        // Otherwise, it's a new schema, so we download it.
        else {

            // Get the file to write the schema on, in case we need it.
            let mut file = BufWriter::new(File::create(&local_schema_path)?);

            // Use Hyper's black magic to download the new schema.
            // NOTE: Github's API has a limit of 1MB per file, so we take it directly from raw.githubusercontent.com instead.
            let mut core = Core::new()?;
            let handle = core.handle();
            let client = Client::configure()
                .connector(hyper_tls::HttpsConnector::new(4, &handle)?)
                .build(&handle);

            let uri = format!("https://raw.githubusercontent.com/Frodo45127/rpfm/master/schemas/{}", schema.schema_file).parse().unwrap();
            let work = client.get(uri).and_then(|res| {

                // Write all to our schema file.
                res.body().for_each(|chunk| {
                    file.write_all(&chunk).map_err(From::from)
                })
            });
            core.run(work)?;
        }
    }

    // Get the local "versions.json" path.
    let versions_path = rpfm_path.to_path_buf().join(PathBuf::from("schemas/versions.json"));

    // Update the "versions.json" to reflect the update.
    let mut file = BufWriter::new(File::create(&versions_path)?);
    file.write_all(serde_json::to_string_pretty(&current_versions)?.as_bytes())?;

    // If we reach this place, return success.
    Ok(())
}
