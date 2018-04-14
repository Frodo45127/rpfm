// Here it goes all the stuff related to the UI part of the "Update Checker" and the future "Autoupdater".
extern crate gtk;
extern crate restson;

use self::restson::RestClient;
use gtk::prelude::*;
use gtk::{ ApplicationWindow, MessageDialog, Statusbar, DialogFlags, MessageType, ButtonsType };

use ui;
use updater::LastestRelease;

/// This enum controls the posible responses from the server.
enum APIResponse {
    SuccessNewUpdate(LastestRelease),
    SuccessNewUpdateHotfix(LastestRelease),
    SuccessNoUpdate,
    SuccessUnknownVersion,
    Error,
}

/// This function checks if there is any newer version of RPFM released. If the `use_dialog` is false,
/// we show the results of the check in the `Statusbar`.
pub fn check_updates(current_version: &str, use_dialog: Option<&ApplicationWindow>, status_bar: Option<&Statusbar>) {

    // Create new client with API base URL
    let mut client = RestClient::new("https://api.github.com").unwrap();
    client.set_header_raw("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:59.0) Gecko/20100101 Firefox/59.0");

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

    // If we want to use a `MessageDialog`...
    if let Some(parent_window) = use_dialog {

        // Get the message we want to show, depending on the result of the "Update Check" from before.
        let message: (String, String) = match apiresponse {
            APIResponse::SuccessNewUpdate(last_release) => (format!("New mayor update found: \"{}\"", last_release.name), format!("Download available here:\n<a href=\"{}\">{}</a>\n\nChanges:\n{}", last_release.html_url, last_release.html_url, last_release.body)),
            APIResponse::SuccessNewUpdateHotfix(last_release) => (format!("New minor update/hotfix found: \"{}\"", last_release.name), format!("Download available here:\n<a href=\"{}\">{}</a>\n\nChanges:\n{}", last_release.html_url, last_release.html_url, last_release.body)),
            APIResponse::SuccessNoUpdate => ("No new updates available".to_owned(), "More luck next time :)".to_owned()),
            APIResponse::SuccessUnknownVersion => ("Error while checking new updates".to_owned(), "There has been a problem when getting the lastest released version number, or the current version number.\n\nThat means I fucked up the last release title. If you see this, please report it here:\n<a href=\"https://github.com/Frodo45127/rpfm/issues\">https://github.com/Frodo45127/rpfm/issues</a>".to_owned()),
            APIResponse::Error => ("Error while checking new updates :(".to_owned(), "If you see this message, there has been a problem with your connection to the Github.com server.\n\nPlease, make sure you can access to <a href=\"https:\\\\api.github.com\">https:\\\\api.github.com</a> and try again.".to_owned()),
        };

        // Create the `MessageDialog` to hold the messages.
        let check_updates_dialog = MessageDialog::new(
            Some(parent_window),
            DialogFlags::from_bits(1).unwrap(),
            MessageType::Info,
            ButtonsType::Close,
            &message.0
        );

        // Show the "Changes" of the release in the `MessageDialog`.
        check_updates_dialog.set_title("Checking for updates...");
        check_updates_dialog.set_property_secondary_use_markup(true);
        check_updates_dialog.set_property_secondary_text(Some(&message.1));

        // Run & Destroy.
        check_updates_dialog.run();
        check_updates_dialog.destroy();
    }

    // If we want to use the `Statusbar`...
    else if let Some(status_bar) = status_bar {

        // Get the message we want to show, depending on the result of the "Update Check" from before.
        let message: String = match apiresponse {
            APIResponse::SuccessNewUpdate(last_release) => format!("New mayor update found: \"{}\".", last_release.name),
            APIResponse::SuccessNewUpdateHotfix(last_release) => format!("New minor update/hotfix found: \"{}\".", last_release.name),
            APIResponse::SuccessNoUpdate => String::from("No new updates available."),
            APIResponse::SuccessUnknownVersion |
            APIResponse::Error => String::from("Error while checking new updates :("),
        };
        ui::show_message_in_statusbar(status_bar, &message);
    }

    // If we reach this place, no valid methods to show the result of the "Update Check" has been provided.
    // So... we do nothing.
    else {}
}
