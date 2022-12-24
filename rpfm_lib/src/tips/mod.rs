//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to deal with tips, or quick notes.
!*/

use getset::{Getters, Setters};
//use git2::{Cred, Direction, ObjectType, Reference, ReferenceFormat, Repository, Signature, StashFlags, build::CheckoutBuilder, RemoteCallbacks, Index, PushOptions};
use serde_derive::{Serialize, Deserialize};

use std::collections::{BTreeMap, HashMap};
use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command as SystemCommand;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{RLibError, Result};
use crate::games::GameInfo;

/// Name of the folder containing all the tips.
const TIPS_LOCAL_FOLDER: &str = "tips/local";
pub const TIPS_REMOTE_FOLDER: &str = "tips/remote";

const TIPS_LOCAL_FILE: &str = "local_tips.json";
const TIPS_REMOTE_FILE: &str = "remote_tips.json";

pub const TIPS_REPO: &str = "https://github.com/Frodo45127/rpfm-messages";
const TIPS_UPLOAD: &str = "https://github.com/RustedLittlePet/rpfm-messages";
const REMOTE_TEMP: &str = "origintemp";
pub const REMOTE: &str = "origin";
pub const MASTER: &str = "master";

const GH_USER: &str = "RustedLittlePet";

/// This one is obtained during compilation.
#[cfg(feature = "support_tip_uploads")]
const TOKEN: &str = include_str!("../../../gh-token");

/// Current structural version of the Tip files, for compatibility purposes.
const CURRENT_STRUCTURAL_VERSION: u16 = 0;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct represents the Tips in memory.
#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Tips {

    /// Local set of tips, stored on disk.
    local_tips: TipSet,

    pack_tips: TipSet,

    /// Remote set of tips, pulled from the repo.
    remote_tips: TipSet,
}

/// Set of tips from a common source.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TipSet {

    /// Version of the file's structure.
    version: u16,

    /// Set of tips split per game, then per path.
    tips: BTreeMap<String, HashMap<String, Vec<Tip>>>,
}

/// Individual tip.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct Tip {

    /// Unique identifier of the tip.
    id: u64,

    /// User that wrote it.
    user: String,

    /// Timestamp of when the tip was created.
    timestamp: u128,

    /// Tip's message.
    message: String,

    /// URL to open when double-clicking the tip.
    url: String,

    /// Path where this tip applies. Empty for global tips.
    path: String,
}

/// This enum controls the possible responses from the server when asking if there is a new tips update.
#[derive(Debug, Serialize, Deserialize)]
pub enum APIResponseTips {
    NewUpdate,
    NoUpdate,
    NoLocalFiles,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of Tips.
impl Tips {

    /// This function loads all the Tips to memory from files in the `tips/` folder.
    pub fn load(local_path: &Path, remote_path: &Path) -> Result<Self> {
        let local_tips = if let Ok(tipset) = TipSet::load(local_path) { tipset } else { TipSet::default() };
        let remote_tips = if let Ok(tipset) = TipSet::load(remote_path) { tipset } else { TipSet::default() };

        Ok(Self {
            local_tips,
            pack_tips: TipSet::default(),
            remote_tips,
        })
    }

    /// This function saves a `Tips` from memory to a file in the `tips/` folder.
    pub fn save(&mut self, path: &Path) -> Result<()> {

        // Make sure the path exists to avoid problems with updating schemas.
        DirBuilder::new().recursive(true).create(path)?;

        let mut file = BufWriter::new(File::create(path)?);
        file.write_all(serde_json::to_string_pretty(&self.local_tips)?.as_bytes())?;

        Ok(())
    }

    /// This function returns the local tip the provided id, if found.
    pub fn get_local_tips_by_id(&self, game_info: &GameInfo, id: u64) -> Option<&Tip> {
        let game_key = game_info.game_key_name();
        if let Some(game_tips) = self.local_tips.tips.get(&game_key) {
            game_tips.iter()
                .find_map(|(_, tips)| tips.iter().find(|tip| tip.id() == &id))
        } else {
            None
        }
    }

    /// This function returns all local tips for the provided path.
    pub fn get_local_tips_for_path(&self, game_info: &GameInfo, path: &[String]) -> Vec<Tip> {
        let game_key = game_info.game_key_name();
        if let Some(game_tips) = self.local_tips.tips.get(&game_key) {
            let path = path.join("/");
            game_tips.iter()
                .filter(|(tip_path, _)| tip_path.starts_with(&path))
                .flat_map(|(_, tips)| tips.to_vec())
                .collect()
        } else {
            vec![]
        }
    }

    /// This function returns all remote tips for the provided path.
    pub fn get_remote_tips_for_path(&self, game_info: &GameInfo, path: &[String]) -> Vec<Tip> {
        let game_key = game_info.game_key_name();
        if let Some(game_tips) = self.remote_tips.tips.get(&game_key) {
            let path = path.join("/");
            game_tips.iter()
                .filter(|(tip_path, _)| tip_path.starts_with(&path))
                .flat_map(|(_, tips)| tips.to_vec())
                .collect()
        } else {
            vec![]
        }
    }

    /// This function adds a tip to the local tips.
    pub fn add_tip_to_local_tips(&mut self, game_info: &GameInfo, tip: Tip) {
        let game_key = game_info.game_key_name();

        // First, delete any tip with the same id, so in case of editing, we "overwrite" the old one instead of creating a new one.
        self.delete_tip_by_id(game_info, *tip.id());

        // Then, add the new tip.
        match self.local_tips.tips.get_mut(&game_key) {
            Some(game_tips) => match game_tips.get_mut(tip.path()) {
                Some(tips) => tips.push(tip),
                None => { game_tips.insert(tip.path().to_owned(), vec![tip]); }
            }
            None => {
                let mut game_tips = HashMap::new();
                game_tips.insert(tip.path().to_owned(), vec![tip]);
                self.local_tips.tips.insert(game_key, game_tips);
            }
        }
    }

    /// This function adds a tip to the remote tips and saves them.
    pub fn add_tip_to_remote_tips(&mut self, game_info: &GameInfo, tip: Tip, save: Option<&Path>) -> Result<()> {
        let game_key = game_info.game_key_name();

        // Add the new tip.
        match self.remote_tips.tips.get_mut(&game_key) {
            Some(game_tips) => match game_tips.get_mut(tip.path()) {
                Some(tips) => tips.push(tip),
                None => { game_tips.insert(tip.path().to_owned(), vec![tip]); }
            }
            None => {
                let mut game_tips = HashMap::new();
                game_tips.insert(tip.path().to_owned(), vec![tip]);
                self.remote_tips.tips.insert(game_key, game_tips);
            }
        }

        if let Some(file_path) = save {

            // Make sure the path exists to avoid problems with updating schemas.
            DirBuilder::new().recursive(true).create(file_path)?;

            let mut file = BufWriter::new(File::create(file_path)?);
            file.write_all(serde_json::to_string_pretty(&self.remote_tips)?.as_bytes())?;
        }

        Ok(())
    }

    /// This function deletes the tip with the corresponding id.
    pub fn delete_tip_by_id(&mut self, game_info: &GameInfo, id: u64) {
        let game_key = game_info.game_key_name();

        if let Some(old_game_tips) = self.local_tips.tips.get_mut(&game_key) {
            old_game_tips.iter_mut().for_each(|(_, tips)| tips.retain(|old_game_tip| old_game_tip.id() != &id));
            old_game_tips.retain(|_, tips| !tips.is_empty());
        }
    }
    /*
    /// This function checks if there is a new tips update in the tips repo.
    pub fn check_update() -> Result<APIResponseTips> {

        let remote_tips_path = get_remote_tips_path()?;
        let mut repo = match Repository::open(&remote_tips_path) {
            Ok(repo) => repo,

            // If this fails, it means we either we don´t have the tips downloaded, or we have a folder without the .git folder.
            Err(_) => return Ok(APIResponseTips::NoLocalFiles),
        };

        // Just in case there are loose changes, stash them.
        // Ignore a fail on this, as it's possible we don't have contents to stash.
        let current_branch_name = Reference::normalize_name(repo.head()?.name().unwrap(), ReferenceFormat::ALLOW_ONELEVEL)?.to_lowercase();
        let master_refname = format!("refs/heads/{}", MASTER);

        let signature = Signature::now("RPFM Updater", "-")?;
        let stash_id = repo.stash_save(&signature, &format!("Stashed changes before checking for updates from branch {}", current_branch_name), Some(StashFlags::INCLUDE_UNTRACKED));

        // In case we're not in master, checkout the master branch.
        if current_branch_name != master_refname {
            repo.set_head(&master_refname)?;
        }

        // Fetch the info of the master branch.
        repo.find_remote(REMOTE)?.fetch(&[MASTER], None, None)?;
        let analysis = {
            let fetch_head = repo.find_reference("FETCH_HEAD")?;
            let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
            repo.merge_analysis(&[&fetch_commit])?
        };

        // Reset the repo to his original state after the check
        if current_branch_name != master_refname {
            let _ = repo.set_head(&current_branch_name);
        }
        if stash_id.is_ok() {
            let _ = repo.stash_pop(0, None);
        }

        if analysis.0.is_up_to_date() {
            Ok(APIResponseTips::NoUpdate)
        }

        // If the branch is a fast-forward, or has diverged, ask for an update.
        else if analysis.0.is_fast_forward() || analysis.0.is_normal() || analysis.0.is_none() || analysis.0.is_unborn() {
            Ok(APIResponseTips::NewUpdate)
        }

        // Otherwise, it means the branches diverged. This may be due to local changes or due to me diverging the master branch with a force push.
        else {
            Err(ErrorKind::MessagesUpdateError.into())
        }
    }

    /// This function downloads the latest revision of the schema repository.
    pub fn update_from_repo() -> Result<()> {
        let remote_tips_path = get_remote_tips_path()?;
        let mut repo = match Repository::open(&remote_tips_path) {
            Ok(repo) => repo,
            Err(_) => {

                // If it fails to open, it means either we don't have the .git folder, or we don't have a folder at all.
                // In either case, recreate it and redownload the messages repo. No more steps are needed here.
                // On windows, remove the read-only flags before doing anything else, or this will fail.
                if cfg!(target_os = "windows") {
                    let path = remote_tips_path.to_string_lossy().to_string() + "\\*.*";
                    let _ = SystemCommand::new("attrib").arg("-r").arg(path).arg("/s").output();
                }
                let _ = std::fs::remove_dir_all(&remote_tips_path);
                DirBuilder::new().recursive(true).create(&remote_tips_path)?;
                match Repository::clone(TIPS_REPO, &remote_tips_path) {
                    Ok(_) => return Ok(()),
                    Err(_) => return Err(ErrorKind::MessagesUpdateError.into()),
                }
            }
        };

        // Just in case there are loose changes, stash them.
        // Ignore a fail on this, as it's possible we don't have contents to stash.
        let current_branch_name = Reference::normalize_name(repo.head()?.name().unwrap(), ReferenceFormat::ALLOW_ONELEVEL)?.to_lowercase();
        let master_refname = format!("refs/heads/{}", MASTER);

        let signature = Signature::now("RPFM Updater", "-")?;
        let stash_id = repo.stash_save(&signature, &format!("Stashed changes before update from branch {}", current_branch_name), Some(StashFlags::INCLUDE_UNTRACKED));

        // In case we're not in master, checkout the master branch.
        if current_branch_name != master_refname {
            repo.set_head(&master_refname)?;
        }

        // If it worked, now we have to do a pull from master. Sadly, git2-rs does not support pull.
        // Instead, we kinda force a fast-forward. Made in StackOverflow.
        repo.find_remote(REMOTE)?.fetch(&[MASTER], None, None)?;
        let (analysis, fetch_commit_id) = {
            let fetch_head = repo.find_reference("FETCH_HEAD")?;
            let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
            (repo.merge_analysis(&[&fetch_commit])?, fetch_commit.id())
        };

        // If we're up to date, nothing more is needed.
        if analysis.0.is_up_to_date() {

            // Reset the repo to his original state after the check
            if current_branch_name != master_refname {
                let _ = repo.set_head(&current_branch_name);
            }
            if stash_id.is_ok() {
                let _ = repo.stash_pop(0, None);
            }
            Err(ErrorKind::NoMessagesUpdatesAvailable.into())
        }

        // If we can do a fast-forward, we do it. This is the preferred option.
        else if analysis.0.is_fast_forward() {
            let mut reference = repo.find_reference(&master_refname)?;
            reference.set_target(fetch_commit_id, "Fast-Forward")?;
            repo.set_head(&master_refname)?;
            repo.checkout_head(Some(CheckoutBuilder::default().force())).map_err(From::from)
        }

        // If not, we face multiple problems:
        // - If there are uncommitted changes: covered by the stash.
        // - If we're not in the branch: covered by the branch switch.
        // - If the branches diverged: this one... the cleanest way to deal with it should be redownload the repo.
        else if analysis.0.is_normal() || analysis.0.is_none() || analysis.0.is_unborn() {

            // On windows, remove the read-only flags before doing anything else, or this will fail.
            if cfg!(target_os = "windows") {
                let path = remote_tips_path.to_string_lossy().to_string() + "\\*.*";
                let _ = SystemCommand::new("attrib").arg("-r").arg(path).arg("/s").output();
            }
            let _ = std::fs::remove_dir_all(&remote_tips_path);
            Self::update_from_repo()
        }
        else {

            // Reset the repo to his original state after the check
            if current_branch_name != master_refname {
                let _ = repo.set_head(&current_branch_name);
            }
            if stash_id.is_ok() {
                let _ = repo.stash_pop(0, None);
            }

            Err(ErrorKind::MessagesUpdateError.into())
        }
    }

    /// This function tries to publish the local tip with the provided id, in the remote repo.
    pub fn publish_tip_by_id(&mut self, id: u64) -> Result<()> {

        // If this isn't enabled, we have no token. Do not try to upload.
        if !cfg!(feature = "support_tip_uploads") {
            return Err(ErrorKind::TipPublishUnsupported.into());
        } else {

            // Open the repo. If it fail's to open, download it.
            let remote_tips_path = get_remote_tips_path()?;
            let mut repo = match Repository::open(&remote_tips_path) {
                Ok(repo) => repo,
                Err(_) => {

                    // If it fails to open, it means either we don't have the .git folder, or we don't have a folder at all.
                    // In either case, recreate it and redownload the messages repo. No more steps are needed here.
                    // On windows, remove the read-only flags before doing anything else, or this will fail.
                    if cfg!(target_os = "windows") {
                        let path = remote_tips_path.to_string_lossy().to_string() + "\\*.*";
                        let _ = SystemCommand::new("attrib").arg("-r").arg(path).arg("/s").output();
                    }
                    let _ = std::fs::remove_dir_all(&remote_tips_path);
                    DirBuilder::new().recursive(true).create(&remote_tips_path)?;
                    match Repository::clone(TIPS_REPO, &remote_tips_path) {
                        Ok(_) => return Ok(()),
                        Err(_) => return Err(ErrorKind::MessagesUpdateError.into()),
                    }
                }
            };

            // Make sure our repo is updated before continuing. This can fail.
            let _ = Self::update_from_repo();

            // TODO: check if the pull request for this new branch already exists and is open. Merged ones will be ignored for the sake of allowing editing tips.

            // Create the new branch for the commit.
            let tip = self.get_local_tips_by_id(id).ok_or_else(|| Error::from(ErrorKind::LocalTipNotFound))?.clone();
            let mut current_branch_name = Reference::normalize_name(repo.head()?.name().unwrap(), ReferenceFormat::ALLOW_ONELEVEL)?.to_lowercase();
            let master_refname = format!("refs/heads/{}", MASTER);
            let id_refname = format!("refs/heads/{}", tip.get_ref_id());

            // Make sure we always start at master.
            if current_branch_name != master_refname {
                let _ = repo.set_head(&master_refname);
                current_branch_name = Reference::normalize_name(repo.head()?.name().unwrap(), ReferenceFormat::ALLOW_ONELEVEL)?.to_lowercase();
            }

            // Stash any changes unrelated with our commit.
            let signature = Signature::now(GH_USER, "-")?;
            let _stash_id = repo.stash_save(&signature, &format!("Stashed changes before update from branch {}", current_branch_name), Some(StashFlags::INCLUDE_UNTRACKED));

            // Make sure our remote is setup properly.
            Self::add_remote(&mut repo);

            // Switch to the new branch.
            if current_branch_name != id_refname {
                let fetch_head = repo.find_reference("FETCH_HEAD")?;
                let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
                repo.branch_from_annotated_commit(&tip.get_ref_id().to_string(), &fetch_commit, true)?;
                repo.set_head(&id_refname)?;
                current_branch_name = Reference::normalize_name(repo.head()?.name().unwrap(), ReferenceFormat::ALLOW_ONELEVEL)?.to_lowercase();
            }

            // Add the tip to the remote tips file.
            self.add_tip_to_remote_tips(tip, true)?;

            // Commit the change and push it.
            let mut index = repo.index()?;
            index.add_path(&PathBuf::from(TIPS_REMOTE_FILE))?;
            let push_result = self.commit_and_push_to_remote(&repo, &mut index, &signature, &id_refname);

            // Reset the repo to master after publish or failure.
            if current_branch_name != master_refname {
                let _stash_id = repo.stash_save(&signature, &format!("Stashed changes before moving back to branch {}", master_refname), Some(StashFlags::INCLUDE_UNTRACKED));
                let _ = repo.set_head(&master_refname);
                let _stash_id = repo.stash_save(&signature, &format!("Stashed changes before moving back to branch {}", master_refname), Some(StashFlags::INCLUDE_UNTRACKED));
            }

            push_result
        }
    }

    /// Add our remote temp to the repo, if it doesn't have it already.
    fn add_remote(repo: &mut Repository) {
        if repo.find_remote(REMOTE_TEMP).is_err() {
            let _ = repo.remote(REMOTE_TEMP, TIPS_UPLOAD);
        }
    }

    /// This function takes care of commiting changes and pushing them to the remote branch.
    ///
    /// All branch-changing logic is out of this function!!!
    fn commit_and_push_to_remote(&self, repo: &Repository, index: &mut Index, signature: &Signature, id_refname: &str) -> Result<()> {

        // Create the new commit over the last one.
        let oid = index.write_tree()?;
        let tree = repo.find_tree(oid)?;
        let parent_commit = repo.head()?.resolve()?.peel(ObjectType::Commit)?.into_commit().unwrap();
        repo.commit(Some("HEAD"), &signature, &signature, "New Tip.", &tree, &[&parent_commit])?;

        // Prepare the connection.
        let mut remote = repo.find_remote(REMOTE_TEMP)?;
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_, _, _| Self::get_credentials());
        remote.connect_auth(Direction::Push, Some(callbacks), None)?;

        let mut push_options = PushOptions::new();
        let mut callbacks = RemoteCallbacks::new();
        let refspecs = format!("{}:{}", id_refname, id_refname);

        callbacks.credentials(|_, _, _| Self::get_credentials());
        push_options.remote_callbacks(callbacks);
        remote.push(&[&refspecs], Some(&mut push_options))?;

        Ok(())
    }

    /// This function gets the credentials needed for push to the remote.
    fn get_credentials() -> std::result::Result<Cred, git2::Error> {
        #[cfg(feature = "support_tip_uploads")] {

            // Make sure there isn't newlines at the end of the token.
            let mut token = TOKEN.to_owned();
            if token.ends_with('\n') {
                token.pop();
            }

            return Cred::userpass_plaintext(GH_USER, &token)
        }

        // If we don't have support for uploads enabled, return invalid credentials.
        #[cfg(not(any(feature = "support_tip_uploads")))] {
            Cred::default()
        }
    }*/
}

/// Implementation of TipSet.
impl TipSet {

    /// This function tries to load a tipset to memory.
    pub fn load(file_path: &Path) -> Result<Self> {
        let file = BufReader::new(File::open(file_path)?);
        serde_json::from_reader(file).map_err(From::from)
    }
}

/// Default implementation for TipSet.
impl Default for TipSet {
    fn default() -> Self {
        Self {
            version: CURRENT_STRUCTURAL_VERSION,
            tips: BTreeMap::new(),
        }
    }
}

/// Default implementation for Tip.
impl Default for Tip {
    fn default() -> Self {
        Self {
            id: rand::random::<u64>(),

            // TODO: MAKE this user be picked automatically from settings.
            user: String::new(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
            message: String::new(),
            url: String::new(),
            path: String::new(),
        }
    }
}
