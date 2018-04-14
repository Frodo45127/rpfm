// This file contains all the stuff needed for the "Update Checker" and for the future "Autoupdater".
extern crate restson;

use self::restson::{RestPath,Error};

#[derive(Serialize,Deserialize,Debug)]
pub struct LastestRelease {
    pub name: String,
    pub html_url: String,
    pub body: String
}

// Path of the REST endpoint: e.g. http://<baseurl>/anything
impl RestPath<()> for LastestRelease {
    fn get_path(_: ()) -> Result<String,Error> {
        Ok(String::from("repos/frodo45127/rpfm/releases/latest"))
    }
}
