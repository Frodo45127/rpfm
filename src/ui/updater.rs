// Here it goes all the stuff related to the UI part of the "Update Checker" and the future "Autoupdater".
extern crate serde_json;
extern crate restson;
extern crate qt_widgets;
extern crate qt_gui;
extern crate qt_core;
extern crate cpp_utils;

use qt_widgets::{widget::Widget, message_box, message_box::MessageBox};

use qt_core::flags::Flags;

use self::restson::RestClient;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;

use VERSION;
use AppUI;
use QString;
use Commands;
use Data;
use common::*;
use common::communications::*;
use ui::*;
use updater::*;

/// This enum controls the posible responses from the server.
#[derive(Serialize, Deserialize)]
pub enum APIResponse {
    SuccessNewUpdate(LastestRelease),
    SuccessNewUpdateHotfix(LastestRelease),
    SuccessNoUpdate,
    SuccessUnknownVersion,
    Error,
}

/// This enum controls the posible responses from the server. The (Versions, Versions) is local, current.
#[derive(Serialize, Deserialize)]
pub enum APIResponseSchema {
    SuccessNewUpdate(Versions, Versions),
    SuccessNoUpdate,
    Error,
}

/// This function checks if there is any newer version of RPFM released. If the `use_dialog` is false,
/// we make the checks in the background, and pop up a dialog only in case there is an update available.
pub fn check_updates(
    app_ui: &AppUI,
    use_dialog: bool,
) {

    // Create a channel to comunicate with the "Network" thread.
    let (sender_net, receiver_ui) = channel();

    // Create the network thread with the "check_update" operation. We pass an empty PathBuf, because we
    // don't really need to the rpfm_path here.
    thread::spawn(move || { network_thread(sender_net, "check_updates", PathBuf::new()); });

    // If we want to use a Dialog to show the full searching process (clicking in the menu button)...
    if use_dialog {

        // Create the dialog to show the response.
        let mut dialog;
        unsafe { dialog = MessageBox::new_unsafe((
            message_box::Icon::Information,
            &QString::from_std_str("Update Checker"),
            &QString::from_std_str("Searching for updates..."),
            Flags::from_int(2097152), // Close button.
            app_ui.window as *mut Widget,
        )); }

        // Set it to be modal, and show it. Don't execute it, just show it.
        dialog.set_modal(true);
        dialog.show();

        // Get the data from the operation...
        let message = match check_api_response(&receiver_ui) {
            (APIResponse::SuccessNewUpdate(last_release),_) => format!("<h4>New major update found: \"{}\"</h4> <p>Download and changelog available here:<br><a href=\"{}\">{}</a></p>", last_release.name, last_release.html_url, last_release.html_url),
            (APIResponse::SuccessNewUpdateHotfix(last_release),_) => format!("<h4>New minor update/hotfix found: \"{}\"</h4> <p>Download and changelog available here:<br><a href=\"{}\">{}</a></p>", last_release.name, last_release.html_url, last_release.html_url),
            (APIResponse::SuccessNoUpdate,_) => "<h4>No new updates available</h4> <p>More luck next time :)</p>".to_owned(),
            (APIResponse::SuccessUnknownVersion,_) => "<h4>Error while checking new updates</h4> <p>There has been a problem when getting the lastest released version number, or the current version number. That means I fucked up the last release title. If you see this, please report it here:\n<a href=\"https://github.com/Frodo45127/rpfm/issues\">https://github.com/Frodo45127/rpfm/issues</a></p>".to_owned(),
            (APIResponse::Error,_) => "<h4>Error while checking new updates :(</h4> <p>If you see this message, there has been a problem with your connection to the Github.com server. Please, make sure you can access to <a href=\"https://api.github.com\">https://api.github.com</a> and try again.</p>".to_owned(),
        };

        // Change the text of the dialog.
        dialog.set_text(&QString::from_std_str(message));

        // Now, execute the dialog.
        dialog.exec();
    }

    // Otherwise, we just wait until we got a response, and only then (and only in case of new update)... we show a dialog.
    else {

        // Depending on the response, we change the text of the dialog in a way or another, or just stop the loop.
        let message: String = match check_api_response(&receiver_ui) {
            (APIResponse::SuccessNewUpdate(last_release),_) => format!("<h4>New major update found: \"{}\"</h4> <p>Download and changelog available here:<br><a href=\"{}\">{}</a></p>", last_release.name, last_release.html_url, last_release.html_url),
            (APIResponse::SuccessNewUpdateHotfix(last_release),_) => format!("<h4>New minor update/hotfix found: \"{}\"</h4> <p>Download and changelog available here:<br><a href=\"{}\">{}</a></p>", last_release.name, last_release.html_url, last_release.html_url),
            _ => return
        };

        // Create the dialog to show the response.
        let mut dialog;
        unsafe { dialog = MessageBox::new_unsafe((
            message_box::Icon::Information,
            &QString::from_std_str("Update Checker"),
            &QString::from_std_str(message),
            Flags::from_int(2097152), // Close button.
            app_ui.window as *mut Widget,
        )); }

        // Set it to be modal, and execute it.
        dialog.set_modal(true);
        dialog.exec();
    }
}

/// This function checks if there is any newer version of RPFM's schemas released. If the `use_dialog`
/// is false, we only show a dialog in case of update available. Useful for checks at start.
pub fn check_schema_updates(
    app_ui: &AppUI,
    use_dialog: bool,
    rpfm_path: &PathBuf,
    sender_qt: &Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: &Rc<RefCell<Receiver<Data>>>,
) {

    // Create a channel to comunicate with the "Network" thread.
    let (sender_net, receiver_ui) = channel();

    // Create the network thread with the "check_schema_update" operation.
    thread::spawn(clone!(rpfm_path => move || { network_thread(sender_net, "check_schema_updates", rpfm_path); }));

    // Create this here so we can later access again to the response from the server.
    let response: (APIResponse, APIResponseSchema);

    // If we want to use a Dialog to show the full searching process (clicking in the menu button)...
    if use_dialog {

        // Create the dialog to show the response.
        let mut dialog;
        unsafe { dialog = MessageBox::new_unsafe((
            message_box::Icon::Information,
            &QString::from_std_str("Update Schema Checker"),
            &QString::from_std_str("Searching for updates..."),
            Flags::from_int(2097152), // Close button.
            app_ui.window as *mut Widget,
        )); }

        // Add a "Update" button with the "Accept" role, disabled by default.
        let update_button = dialog.add_button((&QString::from_std_str("&Update"), message_box::ButtonRole::AcceptRole));
        unsafe { update_button.as_mut().unwrap().set_enabled(false); }

        // Set it to be modal, and show it. Don't execute it, just show it.
        dialog.set_modal(true);
        dialog.show();

        response = check_api_response(&receiver_ui);

        let message: String = match response {
            (_,APIResponseSchema::SuccessNewUpdate(ref local_versions, ref current_versions)) => {

                // In case of new schema, enable the "Update" button.
                unsafe { update_button.as_mut().unwrap().set_enabled(true); }

                // Create the initial message.
                let mut message = "<h4>New schema update available</h4> <table>".to_owned();

                // For each schema supported...
                for (index, schema) in current_versions.schemas.iter().enumerate() {

                    // Start the line.
                    message.push_str("<tr>");

                    // Add the name of the game.
                    message.push_str(&format!("<td>{}:</td>", schema.schema_file));

                    // If the game exist in the local version, show both versions.
                    if let Some(local_schema) = local_versions.schemas.get(index) {
                        message.push_str(&format!("<td>{} => {}</td>", local_schema.version, schema.version));
                    }

                    // Otherwise, it's a new game. Use 0 as his initial version.
                    else { message.push_str(&format!("<td>0 => {}</td>", schema.version))}

                    // End the line.
                    message.push_str("</tr>");
                }

                // Complete the table.
                message.push_str("</table>");

                // Ask if you want to update.
                message.push_str("<p>Do you want to update the schemas?</p>");

                // Return the message.
                message
            }
            (_,APIResponseSchema::SuccessNoUpdate) => "<h4>No new schema updates available</h4> <p>More luck next time :)</p>".to_owned(),
            (_,APIResponseSchema::Error) => "<h4>Error while checking new updates :(</h4> <p>If you see this message, there has been a problem with your connection to the Github.com server. Please, make sure you can access to <a href=\"https://api.github.com\">https://api.github.com</a> and try again.</p>".to_owned(),
        };

        // Change the text of the dialog.
        dialog.set_text(&QString::from_std_str(message));

        // If we hit "Update"...
        if dialog.exec() == 0 {

            // Useless if, but easiest way I know to get local and current version at this point.
            if let APIResponseSchema::SuccessNewUpdate(local_versions, current_versions) = response.1 {

                // Sent to the background thread the order to download the lastest schemas.
                sender_qt.send(Commands::UpdateSchemas).unwrap();
                sender_qt_data.send(Data::VersionsVersions((local_versions, current_versions))).unwrap();

                // Change the text of the dialog and disable the update button.
                dialog.show();
                dialog.set_text(&QString::from_std_str("<p>Downloading updates, don't close this window...</p> <p>This may take a while.</p>"));
                unsafe { update_button.as_mut().unwrap().set_enabled(false); }

                // Get the data from the operation...
                match check_message_validity_tryrecv(&receiver_qt) {
                    
                    // If it's success....
                    Data::Success => show_dialog(app_ui.window, true, "<h4>Schemas updated and reloaded</h4><p>You can continue using RPFM now.</p>"),

                    // If we got an error...
                    Data::Error(error) => show_dialog(app_ui.window, true, error),

                    // In ANY other situation, it's a message problem.
                    _ => panic!(THREADS_MESSAGE_ERROR),
                }
            }
        }
    }

    // Otherwise, we just wait until we got a response, and only then (and only in case of new schema update)... we show a dialog.
    else {

        response = check_api_response(&receiver_ui);

        // Depending on the response, we change the text of the dialog in a way or another.
        let message = match response {
            (_,APIResponseSchema::SuccessNewUpdate(ref local_versions, ref current_versions)) => {

                // Create the initial message.
                let mut message = "<h4>New schema update available</h4> <table>".to_owned();

                // For each schema supported...
                for (index, schema) in current_versions.schemas.iter().enumerate() {

                    // Start the line.
                    message.push_str("<tr>");

                    // Add the name of the game.
                    message.push_str(&format!("<td>{}:</td>", schema.schema_file));

                    // If the game exist in the local version, show both versions.
                    if let Some(local_schema) = local_versions.schemas.get(index) {
                        message.push_str(&format!("<td>{} => {}</td>", local_schema.version, schema.version));
                    }

                    // Otherwise, it's a new game. Use 0 as his initial version.
                    else { message.push_str(&format!("<td>0 => {}</td>", schema.version))}

                    // End the line.
                    message.push_str("</tr>");
                }

                // Complete the table.
                message.push_str("</table>");

                // Ask if you want to update.
                message.push_str("<p>Do you want to update the schemas?</p>");

                // Return the message.
                message
            }
            _ => return
        };

        // Create the dialog to show the response.
        let mut dialog;
        unsafe { dialog = MessageBox::new_unsafe((
            message_box::Icon::Information,
            &QString::from_std_str("Update Schema Checker"),
            &QString::from_std_str(message),
            Flags::from_int(2097152), // Close button.
            app_ui.window as *mut Widget,
        )); }

        // Add a "Update" button with the "Accept" role.
        let update_button = dialog.add_button((&QString::from_std_str("&Update"), message_box::ButtonRole::AcceptRole));

        // Set it to be modal, and execute it.
        dialog.set_modal(true);

        // If we hit "Update"...
        if dialog.exec() == 0 {

            // Useless if, but easiest way I know to get local and current version at this point.
            if let APIResponseSchema::SuccessNewUpdate(local_versions, current_versions) = response.1 {

                // Sent to the background thread the order to download the lastest schemas.
                sender_qt.send(Commands::UpdateSchemas).unwrap();
                sender_qt_data.send(Data::VersionsVersions((local_versions, current_versions))).unwrap();

                // Change the text of the dialog and disable the update button.
                dialog.show();
                dialog.set_text(&QString::from_std_str("<p>Downloading updates, don't close this window...</p> <p>This may take a while.</p>"));
                unsafe { update_button.as_mut().unwrap().set_enabled(false); }

                // Get the data from the operation...
                match check_message_validity_tryrecv(&receiver_qt) {
                    
                    // If it's success....
                    Data::Success => show_dialog(app_ui.window, true, "<h4>Schemas updated and reloaded</h4><p>You can continue using RPFM now.</p>"),

                    // If we got an error...
                    Data::Error(error) => show_dialog(app_ui.window, true, error),

                    // In ANY other situation, it's a message problem.
                    _ => panic!(THREADS_MESSAGE_ERROR),
                }
            }
        }
    }
}

/// This function check network stuff based on what operation we pass it. It REQUIRES to be executed
/// in a different thread.
fn network_thread(
    sender: Sender<(APIResponse, APIResponseSchema)>,
    operation: &str,
    rpfm_path: PathBuf,
) {
    // Act depending on what that message is.
    match operation {

        // When we want to check if there is an update available...
        "check_updates" => {

            // Get a local copy of the current version to work with.
            let current_version = VERSION;

            // Create new client with API base URL
            let mut client = RestClient::new("https://api.github.com").unwrap();
            client.set_header_raw("User-Agent", &format!("RPFM/{}", current_version));

            // Get `https://api.github.com/repos/frodo45127/rpfm/releases/latest` and deserialize the result automatically
            let apiresponse = match client.get(()) {

                // If we received a response from the server...
                Ok(last_release) => {

                    // We get `last_release` into our `last_release`. Redundant, but the compiler doesn't know his type otherwise.
                    let last_release: LastestRelease = last_release;

                    // Get the last version released. This depends on the fact that the releases are called "vX.X.Xwhatever".
                    // We only compare the numbers here (X.X.X), so we have to remove everything else.
                    let mut last_version = last_release.name.to_owned();
                    last_version.remove(0);
                    last_version.split_off(5);

                    // Get the version numbers from our version and from the lastest released version, so we can compare them.
                    let first = (last_version.chars().nth(0).unwrap_or('0').to_digit(10).unwrap_or(0), current_version.chars().nth(0).unwrap_or('0').to_digit(10).unwrap_or(0));
                    let second = (last_version.chars().nth(2).unwrap_or('0').to_digit(10).unwrap_or(0), current_version.chars().nth(2).unwrap_or('0').to_digit(10).unwrap_or(0));
                    let third = (last_version.chars().nth(4).unwrap_or('0').to_digit(10).unwrap_or(0), current_version.chars().nth(4).unwrap_or('0').to_digit(10).unwrap_or(0));

                    // If this is triggered, there has been a problem parsing the current version or the last version released.
                    if first.0 == 0 && second.0 == 0 && third.0 == 0 || first.1 == 0 && second.1 == 0 && third.1 == 0 {
                        APIResponse::SuccessUnknownVersion
                    }

                    // If the current version is different than the last released version...
                    else if last_version != current_version {

                        // If the lastest released version is lesser than the current version...
                        if first.0 < first.1 {

                            // No update. We are using a newer build than the last build released (dev?).
                            APIResponse::SuccessNoUpdate
                        }

                        // If the lastest released version is greater than the current version...
                        else if first.0 > first.1 {

                            // New major update. No more checks needed.
                            APIResponse::SuccessNewUpdate(last_release)
                        }

                        // If the lastest released version the same than the current version...
                        // We check the second number in the versions, and repeat.
                        else if second.0 < second.1 {

                            // No update. We are using a newer build than the last build released (dev?).
                            APIResponse::SuccessNoUpdate
                        }
                        else if second.0 > second.1 {

                            // New major update. No more checks needed.
                            APIResponse::SuccessNewUpdate(last_release)
                        }

                        // If the lastest released version the same than the current version...
                        // We check the last number in the versions, and repeat.
                        else if third.0 < third.1 {

                            // No update. We are using a newer build than the last build released (dev?).
                            APIResponse::SuccessNoUpdate
                        }

                        // If the lastest released version only has the last number higher, is a hotfix.
                        else if third.0 > third.1 {

                            // New major update. No more checks needed.
                            APIResponse::SuccessNewUpdateHotfix(last_release)
                        }

                        // If both versions are the same, it's a tie. We should never be able to reach this,
                        // thanks to the else a few lines below, but better safe than sorry.
                        else {
                            APIResponse::SuccessNoUpdate
                        }
                    }

                    // If both versions are the same, there is no update.
                    else {
                        APIResponse::SuccessNoUpdate
                    }
                }

                // If there has been no response from the server, or it has responded with an error...
                Err(_) => APIResponse::Error,
            };

            // Send the APIResponse, back to the UI thread.
            sender.send((apiresponse, APIResponseSchema::Error)).unwrap();
        }

        // When we want to check if there is a schema's update available...
        "check_schema_updates" => {

            // Create new client with API base URL
            let mut client = RestClient::new("https://raw.githubusercontent.com").unwrap();
            client.set_header_raw("User-Agent", &format!("RPFM/{}", VERSION));

            // Get `https://raw.githubusercontent.com/Frodo45127/rpfm/master/schemas/versions.json` and deserialize the result automatically.
            let apiresponse = match client.get(()) {

                // If we received a response from the server...
                Ok(current_versions) => {

                    // We get `current_versions` into our `current_versions`. Redundant, but the compiler doesn't know his type otherwise.
                    let current_versions: Versions = current_versions;

                    // Get the local versions.
                    let local_versions: Versions = serde_json::from_reader(BufReader::new(File::open(rpfm_path.to_path_buf().join(PathBuf::from("schemas/versions.json"))).unwrap())).unwrap();

                    // If both versions are equal, we have no updates.
                    if current_versions == local_versions { APIResponseSchema::SuccessNoUpdate }

                    // If the local version have more schemas, it has a local indev schema, so don't consider it as "Updates available."
                    else if current_versions.schemas.len() < local_versions.schemas.len() { APIResponseSchema::SuccessNoUpdate }

                    // In any other sisuation, there is an update (or I broke something).
                    else { APIResponseSchema::SuccessNewUpdate(local_versions, current_versions) }
                }

                // If there has been no response from the server, or it has responded with an error...
                Err(_) => APIResponseSchema::Error,
            };

            // Send the APIResponse, back to the UI thread.
            sender.send((APIResponse::Error, apiresponse)).unwrap();
        }

        _ => panic!("Error while receiving the operation, \"{}\" is not a valid operation.", operation),
    }
}
