//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the `GlobalSearch`.

This module contains the code needed to get a `GlobalSeach` over an entire `PackFile`.
!*/

use qt_widgets::header_view::ResizeMode;
use qt_widgets::tree_view::TreeView;

use qt_gui::list::ListStandardItemMutPtr;
use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::qt::{Orientation, SortOrder};
use qt_core::variant::Variant;

use regex::Regex;
use rayon::prelude::*;

use rpfm_error::{ErrorKind, Result};
use rpfm_lib::packfile::{PackFile, PathType};
use rpfm_lib::packedfile::{DecodedData, DecodedPackedFile, PackedFileType};
use rpfm_lib::packedfile::table::{db::DB, loc::Loc};
use rpfm_lib::packedfile::text::Text;
use rpfm_lib::schema::{Definition, Schema, VersionedFile};
use rpfm_lib::SCHEMA;

use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::global_search_ui::GlobalSearchUI;
use crate::QString;
use crate::UI_STATE;

use self::schema::{SchemaMatches, SchemaMatch};
use self::table::{TableMatches, TableMatch};
use self::text::{TextMatches, TextMatch};

mod schema;
mod table;
mod text;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the information needed to perform a global search, and the results of said search.
#[derive(Debug, Clone)]
pub struct GlobalSearch {

    /// Pattern to search.
    pub pattern: String,

    /// Pattern to use when replacing. This is a hard pattern, which means regex is not allowed here.
    pub replace_text: String,

    /// Should the global search be *Case Sensitive*?
    pub case_sensitive: bool,

    /// If the search must be done using regex instead basic matching.
    pub use_regex: bool,

    /// If we should search on DB Tables.
    pub search_on_dbs: bool,

    /// If we should search on Loc Tables.
    pub search_on_locs: bool,

    /// If we should search on Text PackedFiles.
    pub search_on_texts: bool,

    /// If we should search on the currently loaded Schema.
    pub search_on_schema: bool,

    /// Matches on DB Tables.
    pub matches_db: Vec<TableMatches>,

    /// Matches on Loc Tables.
    pub matches_loc: Vec<TableMatches>,

    /// Matches on Text Tables.
    pub matches_text: Vec<TextMatches>,

    /// Matches on Schema definitions.
    pub matches_schema: Vec<SchemaMatches>,
}

/// This enum defines the matching mode of the search. We use `Pattern` by default, and fall back to it
/// if we try to use `Regex` and the provided regex expresion is invalid.
#[derive(Debug, Clone)]
enum MatchingMode {
    Regex(Regex),
    Pattern,
}

//---------------------------------------------------------------p----------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Default` for `GlobalMatch`.
impl Default for GlobalSearch {
    fn default() -> Self {
        Self {
            pattern: "".to_owned(),
            replace_text: "".to_owned(),
            case_sensitive: false,
            use_regex: false,
            search_on_dbs: true,
            search_on_locs: true,
            search_on_texts: true,
            search_on_schema: false,
            matches_db: vec![],
            matches_loc: vec![],
            matches_text: vec![],
            matches_schema: vec![],
        }
    }
}

/// Implementation of `GlobalMatch`.
impl GlobalSearch {

    /// This function performs a search over the parts of a `PackFile` you specify it, storing his results.
    pub fn search(&mut self, pack_file: &mut PackFile) {

        // If we want to use regex and the pattern is invalid, don't search.
        let matching_mode = if self.use_regex {
            if let Ok(regex) = Regex::new(&self.pattern) {
                MatchingMode::Regex(regex)
            }
            else { MatchingMode::Pattern }
        } else { MatchingMode::Pattern };

        // Ensure we don't store results from previous searches.
        self.matches_db = vec![];
        self.matches_loc = vec![];
        self.matches_text = vec![];
        self.matches_schema = vec![];

        // If we got no schema, don't even decode.
        if let Some(ref schema) = *SCHEMA.lock().unwrap() {
            if self.search_on_dbs {
                let mut packed_files = pack_file.get_ref_mut_packed_files_by_type(&PackedFileType::DB);
                self.matches_db = packed_files.par_iter_mut().filter_map(|packed_file| {
                    let path = packed_file.get_path().to_vec();
                    if let Ok(decoded_packed_file) = packed_file.decode_return_ref_no_locks(&schema) {
                        if let DecodedPackedFile::DB(data) = decoded_packed_file {
                            Some(self.search_on_db(&path, &data, &matching_mode))
                        } else { None }
                    } else { None }
                }).collect();
            }

            if self.search_on_locs {
                let mut packed_files = pack_file.get_ref_mut_packed_files_by_type(&PackedFileType::Loc);
                self.matches_loc = packed_files.par_iter_mut().filter_map(|packed_file| {
                    let path = packed_file.get_path().to_vec();
                    if let Ok(decoded_packed_file) = packed_file.decode_return_ref_no_locks(&schema) {
                        if let DecodedPackedFile::Loc(data) = decoded_packed_file {
                            Some(self.search_on_loc(&path, &data, &matching_mode))
                        } else { None }
                    } else { None }
                }).collect();
            }

            if self.search_on_texts {
                let mut packed_files = pack_file.get_ref_mut_packed_files_by_type(&PackedFileType::Text);
                self.matches_text = packed_files.par_iter_mut().filter_map(|packed_file| {
                    let path = packed_file.get_path().to_vec();
                    if let Ok(decoded_packed_file) = packed_file.decode_return_ref_no_locks(&schema) {
                        if let DecodedPackedFile::Text(data) = decoded_packed_file {
                            Some(self.search_on_text(&path, &data, &matching_mode))
                        } else { None }
                    } else { None }
                }).collect();
            }

            if self.search_on_schema {
                self.search_on_schema(schema, &matching_mode);
            }
        }
    }

    /// This function performs a limited search on the `PackedFiles` in the provided paths, and updates the `GlobalSearch` with the results.
    ///
    /// This means that, as long as you change any `PackedFile` in the `PackFile`, you should trigger this. That way, the `GlobalSearch`
    /// will always be up-to-date in an efficient way.
    ///
    /// If you passed the entire `PackFile` to this and it crashed, it's not an error. I forced that crash. If you want to do that,
    /// use the normal search function, because it's a lot more efficient than this one.
    ///
    /// NOTE: The schema search is not updated on schema change. Remember that.
    pub fn update(&mut self, pack_file: &mut PackFile, updated_paths: &[PathType]) {

        // Don't do anything if we have no pattern to search.
        if &self.pattern == "" { return }

        // If we want to use regex and the pattern is invalid, don't search.
        let matching_mode = if self.use_regex {
            if let Ok(regex) = Regex::new(&self.pattern) {
                MatchingMode::Regex(regex)
            }
            else { MatchingMode::Pattern }
        } else { MatchingMode::Pattern };

        // Turn all our updated packs into `PackedFile` paths, and get them.
        let mut paths = vec![];
        for path_type in updated_paths {
            match path_type {
                PathType::File(path) => paths.push(path.to_vec()),
                PathType::Folder(path) => paths.append(&mut pack_file.get_ref_packed_files_by_path_start(path).iter().map(|x| x.get_path().to_vec()).collect()),
                _ => unimplemented!()
            }
        }

        // We remove the added/edited/deleted files from all the search.
        for path in &paths {
            self.matches_db.retain(|x| &x.path != path);
            self.matches_loc.retain(|x| &x.path != path);
            self.matches_text.retain(|x| &x.path != path);
        }

        // If we got no schema, don't even decode.
        if let Some(ref schema) = *SCHEMA.lock().unwrap() {
            for path in &paths {
                if let Some(packed_file) = pack_file.get_ref_mut_packed_file_by_path(&path) {
                    match packed_file.decode_return_ref_no_locks(&schema).unwrap_or_else(|_| &DecodedPackedFile::Unknown) {
                        DecodedPackedFile::DB(data) => {
                            if self.search_on_dbs {
                                self.search_on_db(&path, data, &matching_mode);
                            }
                        }
                        DecodedPackedFile::Loc(data) => {
                            if self.search_on_locs {
                                self.search_on_loc(&path, data, &matching_mode);
                            }
                        }
                        DecodedPackedFile::Text(data) => {
                            if self.search_on_texts {
                                self.search_on_text(&path, data, &matching_mode);
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }
    }

    /// This function clears the Global Search resutl's data, and reset the UI for it.
    pub fn clear(&mut self, ui: &GlobalSearchUI) {
        self.matches_db = vec![];
        self.matches_loc = vec![];
        self.matches_text = vec![];
        self.matches_schema = vec![];

        unsafe { ui.global_search_matches_db_tree_model.as_mut().unwrap().clear() };
        unsafe { ui.global_search_matches_loc_tree_model.as_mut().unwrap().clear() };
        unsafe { ui.global_search_matches_text_tree_model.as_mut().unwrap().clear() };
        unsafe { ui.global_search_matches_schema_tree_model.as_mut().unwrap().clear() };
    }

    /// This function performs a replace operation over the entire match set, except schemas..
    pub fn replace_all(&mut self, pack_file: &mut PackFile) -> Vec<Vec<String>> {
        let mut errors = vec![];

        // If we want to use regex and the pattern is invalid, don't search.
        let matching_mode = if self.use_regex {
            if let Ok(regex) = Regex::new(&self.pattern) {
                MatchingMode::Regex(regex)
            }
            else { MatchingMode::Pattern }
        } else { MatchingMode::Pattern };

        if let Some(ref schema) = *SCHEMA.lock().unwrap() {
            let mut changed_files = vec![];

            for match_table in &self.matches_db {
                if let Some(packed_file) = pack_file.get_ref_mut_packed_file_by_path(&match_table.path) {
                    if let Ok(packed_file) = packed_file.decode_return_ref_mut_no_locks(&schema) {
                        if let DecodedPackedFile::DB(ref mut table) = packed_file {
                            let mut data = table.get_table_data();
                            for match_data in &match_table.matches {

                                // If any replace in the table fails, forget about this one and try the next one.
                                if self.replace_match_table(&mut data, &mut changed_files, match_table, match_data, &matching_mode).is_err() {
                                    changed_files.retain(|x| x != &match_table.path);
                                    errors.push(match_table.path.to_vec());
                                    break;
                                }
                            }

                            if changed_files.contains(&match_table.path) {
                                if table.set_table_data(&data).is_err() {
                                    changed_files.retain(|x| x != &match_table.path);
                                    errors.push(match_table.path.to_vec());
                                }
                            }
                        }
                    }
                }
            }

            for match_table in &self.matches_loc {
                if let Some(packed_file) = pack_file.get_ref_mut_packed_file_by_path(&match_table.path) {
                    if let Ok(packed_file) = packed_file.decode_return_ref_mut_no_locks(&schema) {
                        if let DecodedPackedFile::Loc(ref mut table) = packed_file {
                            let mut data = table.get_table_data();
                            for match_data in &match_table.matches {

                                // If any replace in the table fails, forget about this one and try the next one.
                                if self.replace_match_table(&mut data, &mut changed_files, match_table, match_data, &matching_mode).is_err() {
                                    changed_files.retain(|x| x != &match_table.path);
                                    errors.push(match_table.path.to_vec());
                                    break;
                                }
                            }

                            if changed_files.contains(&match_table.path) {
                                if table.set_table_data(&data).is_err() {
                                    changed_files.retain(|x| x != &match_table.path);
                                    errors.push(match_table.path.to_vec());
                                }
                            }
                        }
                    }
                }
            }

            let changed_files = changed_files.iter().map(|x| PathType::File(x.to_vec())).collect::<Vec<PathType>>();
            self.update(pack_file, &changed_files);
        }

        errors
    }

    /// This function tries to replace data in a Table PackedFile. It fails if the data is not suitable for that column.
    fn replace_match_table(
        &self,
        data: &mut Vec<Vec<DecodedData>>,
        changed_files: &mut Vec<Vec<String>>,
        match_table: &TableMatches,
        match_data: &TableMatch,
        matching_mode: &MatchingMode,
    ) -> Result<()> {
        if let Some(row) = data.get_mut((match_data.row_number - 1) as usize) {
            if let Some(field) = row.get_mut(match_data.column_number as usize) {
                match field {
                    DecodedData::Boolean(ref mut field) => {
                        let mut string = field.to_string();
                        self.replace_match(&mut string, matching_mode);
                        *field = &string == "true";
                    }
                    DecodedData::Float(ref mut field) => {
                        let mut string = field.to_string();
                        self.replace_match(&mut string, matching_mode);
                        *field = string.parse::<f32>()?;
                    }
                    DecodedData::Integer(ref mut field) => {
                        let mut string = field.to_string();
                        self.replace_match(&mut string, matching_mode);
                        *field = string.parse::<i32>()?;
                    }
                    DecodedData::LongInteger(ref mut field) => {
                        let mut string = field.to_string();
                        self.replace_match(&mut string, matching_mode);
                        *field = string.parse::<i64>()?;
                    }
                    DecodedData::StringU8(ref mut field) |
                    DecodedData::StringU16(ref mut field) |
                    DecodedData::OptionalStringU8(ref mut field) |
                    DecodedData::OptionalStringU16(ref mut field) => self.replace_match(field, matching_mode),
                    DecodedData::Sequence(_) => return Err(ErrorKind::Generic)?,
                }

                if !changed_files.contains(&match_table.path) {
                    changed_files.push(match_table.path.to_vec());
                }
            }
        }

        Ok(())
    }

    /// This function replaces all the matches in the provided text.
    fn replace_match(&self, text: &mut String, matching_mode: &MatchingMode) {
        match matching_mode {
            MatchingMode::Regex(regex) => {
                if regex.is_match(&text) {
                    *text = regex.replace_all(&text, &*self.replace_text).to_string();
                }
            }
            MatchingMode::Pattern => {
                while let Some(start) = text.find(&self.pattern) {
                    let end = start + self.pattern.len();
                    text.replace_range(start..end, &self.replace_text);
                }
            }
        }
    }

    /// This function takes care of loading the results of a global search of `TableMatches` into a model.
    pub fn load_table_matches_to_ui(model: &mut StandardItemModel, tree_view: &mut TreeView, matches: &[TableMatches]) {
        if !matches.is_empty() {

            for match_table in matches {
                if !match_table.matches.is_empty() {
                    let path = match_table.path.join("/");
                    let mut qlist_daddy = ListStandardItemMutPtr::new(());
                    let mut file = StandardItem::new(());
                    let mut fill1 = StandardItem::new(());
                    let mut fill2 = StandardItem::new(());
                    let mut fill3 = StandardItem::new(());
                    file.set_text(&QString::from_std_str(&path));
                    file.set_editable(false);
                    fill1.set_editable(false);
                    fill2.set_editable(false);
                    fill3.set_editable(false);

                    for match_row in &match_table.matches {

                        // Create a new list of StandardItem.
                        let mut qlist_boi = ListStandardItemMutPtr::new(());

                        // Create an empty row.
                        let mut column_name = StandardItem::new(());
                        let mut column_number = StandardItem::new(());
                        let mut row = StandardItem::new(());
                        let mut text = StandardItem::new(());

                        column_name.set_text(&QString::from_std_str(&match_row.column_name));
                        column_number.set_data((&Variant::new2(match_row.column_number), 2));
                        row.set_data((&Variant::new2(match_row.row_number + 1), 2));
                        text.set_text(&QString::from_std_str(&match_row.contents));

                        column_name.set_editable(false);
                        column_number.set_editable(false);
                        row.set_editable(false);
                        text.set_editable(false);

                        // Add an empty row to the list.
                        unsafe { qlist_boi.append_unsafe(&column_name.into_raw()); }
                        unsafe { qlist_boi.append_unsafe(&row.into_raw()); }
                        unsafe { qlist_boi.append_unsafe(&text.into_raw()); }
                        unsafe { qlist_boi.append_unsafe(&column_number.into_raw()); }

                        // Append the new row.
                        file.append_row(&qlist_boi);
                    }
                    unsafe { qlist_daddy.append_unsafe(&file.into_raw()); }
                    unsafe { qlist_daddy.append_unsafe(&fill1.into_raw()); }
                    unsafe { qlist_daddy.append_unsafe(&fill2.into_raw()); }
                    unsafe { qlist_daddy.append_unsafe(&fill3.into_raw()); }
                    model.append_row(&qlist_daddy);
                }
            }

            model.set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("PackedFile/Column"))));
            model.set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Row"))));
            model.set_header_data((2, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Match"))));

            // Hide the column number column for tables.
            tree_view.hide_column(3);
            tree_view.sort_by_column((0, SortOrder::Ascending));

            unsafe { tree_view.header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
        }
    }

    /// This function takes care of loading the results of a global search of `TextMatches` into a model.
    pub fn load_text_matches_to_ui(model: &mut StandardItemModel, tree_view: &mut TreeView, matches: &[TextMatches]) {
        if !matches.is_empty() {
            for match_text in matches {
                if !match_text.matches.is_empty() {
                    let path = match_text.path.join("/");
                    let mut qlist_daddy = ListStandardItemMutPtr::new(());
                    let mut file = StandardItem::new(());
                    let mut fill1 = StandardItem::new(());
                    let mut fill2 = StandardItem::new(());
                    let mut fill3 = StandardItem::new(());
                    file.set_text(&QString::from_std_str(&path));
                    file.set_editable(false);
                    fill1.set_editable(false);
                    fill2.set_editable(false);
                    fill3.set_editable(false);

                    for match_row in &match_text.matches {

                        // Create a new list of StandardItem.
                        let mut qlist_boi = ListStandardItemMutPtr::new(());

                        // Create an empty row.
                        let mut text = StandardItem::new(());
                        let mut row = StandardItem::new(());
                        let mut column = StandardItem::new(());
                        let mut len = StandardItem::new(());

                        text.set_text(&QString::from_std_str(&match_row.text));
                        row.set_data((&Variant::new0(match_row.row + 1), 2));
                        column.set_data((&Variant::new0(match_row.column), 2));
                        len.set_data((&Variant::new2(match_row.len), 2));

                        text.set_editable(false);
                        row.set_editable(false);
                        column.set_editable(false);
                        len.set_editable(false);

                        // Add an empty row to the list.
                        unsafe { qlist_boi.append_unsafe(&text.into_raw()); }
                        unsafe { qlist_boi.append_unsafe(&row.into_raw()); }
                        unsafe { qlist_boi.append_unsafe(&column.into_raw()); }
                        unsafe { qlist_boi.append_unsafe(&len.into_raw()); }

                        // Append the new row.
                        file.append_row(&qlist_boi);
                    }
                    unsafe { qlist_daddy.append_unsafe(&file.into_raw()); }
                    unsafe { qlist_daddy.append_unsafe(&fill1.into_raw()); }
                    unsafe { qlist_daddy.append_unsafe(&fill2.into_raw()); }
                    unsafe { qlist_daddy.append_unsafe(&fill3.into_raw()); }
                    model.append_row(&qlist_daddy);
                }
            }

            model.set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("PackedFile/Text"))));
            model.set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Row"))));
            model.set_header_data((2, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Column"))));
            model.set_header_data((3, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Length"))));

            // Hide the column and lenght numbers on the TreeView.
            tree_view.hide_column(2);
            tree_view.hide_column(3);
            tree_view.sort_by_column((0, SortOrder::Ascending));

            unsafe { tree_view.header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
        }
    }

    /// This function takes care of loading the results of a global search of `SchemaMatches` into a model.
    pub fn load_schema_matches_to_ui(model: &mut StandardItemModel, tree_view: &mut TreeView, matches: &[SchemaMatches]) {
        if !matches.is_empty() {

            for match_schema in matches {
                if !match_schema.matches.is_empty() {
                    let mut qlist_daddy = ListStandardItemMutPtr::new(());
                    let mut versioned_file = StandardItem::new(());
                    let mut fill1 = StandardItem::new(());
                    let mut fill2 = StandardItem::new(());

                    let name = if let Some(ref name) = match_schema.versioned_file_name {
                        format!("{}/{}", match_schema.versioned_file_type, name)
                    } else { format!("{}", match_schema.versioned_file_type) };

                    versioned_file.set_text(&QString::from_std_str(&name));
                    versioned_file.set_editable(false);
                    fill1.set_editable(false);
                    fill2.set_editable(false);

                    for match_row in &match_schema.matches {

                        // Create a new list of StandardItem.
                        let mut qlist_boi = ListStandardItemMutPtr::new(());

                        // Create an empty row.
                        let mut name = StandardItem::new(());
                        let mut version = StandardItem::new(());
                        let mut column = StandardItem::new(());

                        name.set_text(&QString::from_std_str(&match_row.name));
                        version.set_data((&Variant::new0(match_row.version), 2));
                        column.set_data((&Variant::new2(match_row.column), 2));

                        name.set_editable(false);
                        version.set_editable(false);
                        column.set_editable(false);

                        // Add an empty row to the list.
                        unsafe { qlist_boi.append_unsafe(&name.into_raw()); }
                        unsafe { qlist_boi.append_unsafe(&version.into_raw()); }
                        unsafe { qlist_boi.append_unsafe(&column.into_raw()); }

                        // Append the new row.
                        versioned_file.append_row(&qlist_boi);
                    }
                    unsafe { qlist_daddy.append_unsafe(&versioned_file.into_raw()); }
                    unsafe { qlist_daddy.append_unsafe(&fill1.into_raw()); }
                    unsafe { qlist_daddy.append_unsafe(&fill2.into_raw()); }
                    model.append_row(&qlist_daddy);
                }
            }

            model.set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("VersionedFile (Type, Name)/Column Name"))));
            model.set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Definition Version"))));
            model.set_header_data((2, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Column Index"))));

            // Hide the column number column for tables.
            tree_view.hide_column(2);
            tree_view.sort_by_column((0, SortOrder::Ascending));

            unsafe { tree_view.header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
        }
    }

    /// This function takes care of updating the results of a global search.
    ///
    /// It's here instead of in a slot because we need to pass the paths to update to it.
    pub fn update_matches_ui(ui: &GlobalSearchUI, paths: Vec<PathType>) {

        // Create the global search and populate it with all the settings for the search.
        let global_search = (*UI_STATE.global_search.read().unwrap()).clone();

        CENTRAL_COMMAND.send_message_qt(Command::GlobalSearchUpdate(global_search, paths));

        // While we wait for an answer, we need to clear the current results panels.
        let tree_view_db = unsafe { ui.global_search_matches_db_tree_view.as_mut().unwrap() };
        let tree_view_loc = unsafe { ui.global_search_matches_loc_tree_view.as_mut().unwrap() };

        let model_db = unsafe { ui.global_search_matches_db_tree_model.as_mut().unwrap() };
        let model_loc = unsafe { ui.global_search_matches_loc_tree_model.as_mut().unwrap() };

        model_db.clear();
        model_loc.clear();

        match CENTRAL_COMMAND.recv_message_qt() {
            Response::GlobalSearch(global_search) => {

                // Load the results to their respective models. Then, store the GlobalSearch for future checks.
                GlobalSearch::load_table_matches_to_ui(model_db, tree_view_db, &global_search.matches_db);
                GlobalSearch::load_table_matches_to_ui(model_loc, tree_view_loc, &global_search.matches_loc);
            }

            // In ANY other situation, it's a message problem.
            _ => panic!(THREADS_COMMUNICATION_ERROR)
        }
    }

    /// This function performs a search over the provided DB Table.
    fn search_on_db(&self, path: &[String], table_data: &DB, matching_mode: &MatchingMode) -> TableMatches {
        let mut matches = TableMatches::new(path);

        for (row_number, row) in table_data.get_ref_table_data().iter().enumerate() {
            for (column_number, cell) in row.iter().enumerate() {
                match cell {
                    DecodedData::Boolean(ref data) => {
                        let text = if *data { "true" } else { "false" };
                        self.match_decoded_data(text, matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64);
                    }
                    DecodedData::Float(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::Integer(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::LongInteger(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),

                    DecodedData::StringU8(ref data) |
                    DecodedData::StringU16(ref data) |
                    DecodedData::OptionalStringU8(ref data) |
                    DecodedData::OptionalStringU16(ref data) => self.match_decoded_data(data, matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::Sequence(_) => continue,
                }
            }
        }

        matches
    }

    /// This function performs a search over the provided Loc Table.
    fn search_on_loc(&self, path: &[String], table_data: &Loc, matching_mode: &MatchingMode) -> TableMatches {
        let mut matches = TableMatches::new(path);

        for (row_number, row) in table_data.get_ref_table_data().iter().enumerate() {
            for (column_number, cell) in row.iter().enumerate() {
                match cell {
                    DecodedData::Boolean(ref data) => {
                        let text = if *data { "true" } else { "false" };
                        self.match_decoded_data(text, matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64);
                    }
                    DecodedData::Float(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::Integer(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::LongInteger(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),

                    DecodedData::StringU8(ref data) |
                    DecodedData::StringU16(ref data) |
                    DecodedData::OptionalStringU8(ref data) |
                    DecodedData::OptionalStringU16(ref data) => self.match_decoded_data(data, matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::Sequence(_) => continue,
                }
            }
        }

        matches
    }

    /// This function performs a search over the provided Text PackedFile.
    fn search_on_text(&self, path: &[String], data: &Text, matching_mode: &MatchingMode) -> TextMatches {
        let mut matches = TextMatches::new(path);
        match matching_mode {
            MatchingMode::Regex(regex) => {
                for (row, data) in data.get_ref_contents().lines().enumerate() {
                    for match_data in regex.find_iter(data) {
                        matches.matches.push(
                            TextMatch::new(
                                match_data.start() as u64,
                                row as u64,
                                (match_data.end() - match_data.start()) as i64,
                                data.to_owned()
                            )
                        );
                    }
                }
            }

            // If we're searching a pattern, we just check every text PackedFile, line by line.
            MatchingMode::Pattern => {
                let lenght = self.pattern.chars().count();
                let mut column = 0;

                for (row, data) in data.get_ref_contents().lines().enumerate() {
                    loop {
                        match data.get(column..) {
                            Some(text) => {
                                match text.find(&self.pattern) {
                                    Some(position) => {
                                        matches.matches.push(TextMatch::new(position as u64, row as u64, lenght as i64, data.to_owned()));
                                        column += position + lenght;
                                    }
                                    None => break,
                                }
                            }
                            None => break,
                        }
                    }

                    column = 0;
                }
            }
        }

        matches
    }


    /// This function performs a search over the provided Text PackedFile.
    fn search_on_schema(&mut self, schema: &Schema, matching_mode: &MatchingMode) {
        for versioned_file in schema.get_ref_versioned_file_all() {
            let mut matches = vec![];
            match versioned_file {
                VersionedFile::DB(_, definitions) |
                VersionedFile::Loc(definitions) |
                VersionedFile::DepManager(definitions)  => {

                    match matching_mode {
                        MatchingMode::Regex(regex) => {
                            for definition in definitions {
                                for (index, field) in definition.fields.iter().enumerate() {
                                    if regex.is_match(&field.name) {
                                        matches.push(SchemaMatch::new(
                                            definition.version,
                                            index as u32,
                                            field.name.to_owned()
                                        ));
                                    }
                                }
                            }
                        }

                        // If we're searching a pattern, we just check every text PackedFile, line by line.
                        MatchingMode::Pattern => {
                            for definition in definitions {
                                for (index, field) in definition.fields.iter().enumerate() {
                                    if field.name.contains(&self.pattern) {
                                        matches.push(SchemaMatch::new(
                                            definition.version,
                                            index as u32,
                                            field.name.to_owned()
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !matches.is_empty() {
                let (versioned_file_type, versioned_file_name) = match versioned_file {
                    VersionedFile::DB(name, _) => ("DB".to_owned(), Some(name.to_owned())),
                    VersionedFile::Loc(_) => ("Loc".to_owned(), None),
                    VersionedFile::DepManager(_) => ("Dependency Manager".to_owned(), None),
                };
                let mut schema_matches = SchemaMatches::new(versioned_file_type, versioned_file_name);
                schema_matches.matches = matches;
                self.matches_schema.push(schema_matches);
            }
        }
    }


    /// This function check if the provided `&str` matches our search.
    fn match_decoded_data(
        &self,
        text: &str,
        matching_mode: &MatchingMode,
        matches: &mut Vec<TableMatch>,
        definition: &Definition,
        column_number: u32,
        row_number: i64,
    ) {
        match matching_mode {
            MatchingMode::Regex(regex) => {
                if regex.is_match(&text) {
                    let column_name = &definition.fields[column_number as usize].name;
                    matches.push(TableMatch::new(&column_name, column_number, row_number, text));
                }
            }

            MatchingMode::Pattern => {
                if text.contains(&self.pattern) {
                    let column_name = &definition.fields[column_number as usize].name;
                    matches.push(TableMatch::new(column_name, column_number, row_number, text));
                }
            }
        }
    }
}
