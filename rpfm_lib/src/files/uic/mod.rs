//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write binary UIC (UI Component) files.
//!
//! UIC files define the layout and functionality of the UI. Binaries until 3k.
//! From there onwards they're in xml format.
//!
//! Unifinished module, do not use.

use serde_derive::{Serialize, Deserialize};

use std::collections::HashMap;
use std::io::SeekFrom;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};

pub const BASE_PATH: &str = "ui";

/// Extension of UIC files in some games (they don't have extensions in some games).
pub const EXTENSIONS: [&str; 2] = [".cml", ".xml"];

mod xml;

//#[cfg(test)] mod uic_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire UI Component decoded in memory.
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct UIC {
    version: u32,
    source_is_xml: bool,
    comment: String,
    precache_condition: String,
    hierarchy: HashMap<String, HierarchyItem>,
    components: HashMap<String, Component>,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct HierarchyItem {
    this: String,
    childs: HashMap<String, HierarchyItem>,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Component {
    this: String,
    id: String,
    allowhorizontalresize: Option<bool>,
    priority: Option<u8>,
    tooltipslocalised: Option<bool>,
    uniqueguid: String, // Same as this
    update_when_not_visible: Option<bool>,
    current_state: Option<String>, //="B99323DE-629E-4A03-B0AC63E5D0C26CCC"
    default_state: Option<String>, //="B99323DE-629E-4A03-B0AC63E5D0C26CCC">

    callbackwithcontextlist: Option<HashMap<String, CallbackWithContext>>,
    componentimages: Option<HashMap<String, ComponentImage>>,
    states: Option<HashMap<String, State>>,
    animations: Option<HashMap<String, Animation>>,
    layout_engine: Option<LayoutEngine>,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct CallbackWithContext {
    callback_id: String,
    context_object_id: Option<String>,
    context_function_id: Option<String>,

    child_m_user_properties: Option<HashMap<String, Property>>,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Property {
    name: String,
    value: String,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct ComponentImage {
    this: String,
    uniqueguid: String,
    imagepath: Option<String>,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    this: String,
    name: String,
    width: Option<u32>,
    text: Option<String>,
    text_h_align: Option<String>,
    text_y_offset: Option<String>,
    text_h_behaviour: Option<String>,
    text_localised: Option<bool>,
    text_label: Option<String>,
    font_m_font_name: Option<String>,
    font_m_size: Option<u8>,
    font_m_colour: Option<String>,
    font_m_leading: Option<u8>,
    fontcat_name: Option<String>,
    interactive: Option<bool>,
    uniqueguid: String, // Same as this
    imagemetrics: Option<HashMap<String, Image>>,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Image {
    this: String,
    uniqueguid: String,
    componentimage: String,
    offset: Option<String>,
    colour: Option<String>,
    dockpoint: Option<String>,
    dock_offset: Option<String>,
    canresizeheight: Option<bool>,
    canresizewidth: Option<bool>,
    ui_colour_preset_type_key: Option<String>,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Animation {
    id: String,
    frames: Vec<Frame>,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Frame {
    interpolationtime: Option<u32>,
    interpolationpropertymask: Option<u8>,
    targetmetrics_m_height: Option<i32>,
    targetmetrics_m_width: Option<i32>
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct LayoutEngine {
    r#type: String,
    spacing: String,
    sizetocontent: bool,
    margins: String,
    columnwidths: Vec<i32>,
}

//---------------------------------------------------------------------------//
//                           Implementation of Text
//---------------------------------------------------------------------------//

impl Decodeable for UIC {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut data_local = vec![];
        let read = data.read_to_end(&mut data_local)?;
        data.seek(SeekFrom::Current(-(read as i64)))?;

        // TODO: Unhardcode this version.
        if content_inspector::inspect(&data_local).is_text() {
            Ok(Self::from(xml::v138::XmlLayout::decode(data, extra_data)?))
        } else {
            todo!()
        }
    }
}

impl Encodeable for UIC {

    fn encode<W: WriteBytes>(&mut self, _buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        if self.source_is_xml {
            //xml::v138::XmlLayout::encode(&mut self, buffer, extra_data)
            todo!()
        } else {
            todo!()
        }
    }
}
