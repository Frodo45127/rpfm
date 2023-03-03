//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write UIC layouts, v138, xml format.
//!
//! For internal use only.

use serde_derive::{Serialize, Deserialize};

use std::collections::HashMap;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable, uic::UIC};

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename = "layout")]
pub struct XmlLayout {
    version: u32,
    comment: String,
    precache_condition: String,
    hierarchy: HashMap<String, XmlHierarchyItem>,
    components: HashMap<String, XmlComponent>,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct XmlHierarchyItem {
    this: String,
    #[serde(rename = "$value")] childs: Option<Vec<Self>>,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct XmlComponent {
    this: String,
    id: String,
    allowhorizontalresize: Option<bool>,
    priority: Option<u8>,
    tooltipslocalised: Option<bool>,
    uniqueguid: String, // Same as this
    updatewhennotvisible: Option<bool>,
    currentstate: Option<String>, //="B99323DE-629E-4A03-B0AC63E5D0C26CCC"
    defaultstate: Option<String>, //="B99323DE-629E-4A03-B0AC63E5D0C26CCC">

    callbackwithcontextlist: Option<HashMap<String, XmlCallbackWithContext>>,
    componentimages: Option<HashMap<String, XmlComponentImage>>,
    states: Option<HashMap<String, XmlState>>,
    animations: Option<HashMap<String, XmlAnimation>>,
    #[serde(rename = "LayoutEngine")] layout_engine: Option<XmlLayoutEngine>,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct XmlCallbackWithContext {
    callback_id: String,
    context_object_id: Option<String>,
    context_function_id: Option<String>,

    child_m_user_properties: Option<HashMap<String, XmlProperty>>,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct XmlProperty {
    name: String,
    value: String,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct XmlComponentImage {
    this: String,
    uniqueguid: String,
    imagepath: Option<String>,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct XmlState {
    this: String,
    name: String,
    width: Option<u32>,
    text: Option<String>,
    texthalign: Option<String>,
    textyoffset: Option<String>,
    texthbehaviour: Option<String>,
    textlocalised: Option<bool>,
    textlabel: Option<String>,
    font_m_font_name: Option<String>,
    font_m_size: Option<u8>,
    font_m_colour: Option<String>,
    font_m_leading: Option<u8>,
    fontcat_name: Option<String>,
    interactive: Option<bool>,
    uniqueguid: String, // Same as this
    imagemetrics: Option<HashMap<String, XmlImage>>,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct XmlImage {
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
pub struct XmlAnimation {
    id: String,
    frames: Vec<XmlFrame>,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct XmlFrame {
    interpolationtime: Option<u32>,
    interpolationpropertymask: Option<u8>,
    targetmetrics_m_height: Option<i32>,
    targetmetrics_m_width: Option<i32>
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct XmlLayoutEngine {
    #[serde(rename = "type")] layout_type: String,
    spacing: Option<String>,
    sizetocontent: bool,
    margins: Option<String>,
    columnwidths: Option<HashMap<String, XmlLayoutEngineColumn>>,
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct XmlLayoutEngineColumn {
    width: i32,
}

//---------------------------------------------------------------------------//
//                           Implementation
//---------------------------------------------------------------------------//

impl Decodeable for XmlLayout {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        serde_xml_rs::from_reader(data).map_err(From::from)
    }
}

impl Encodeable for XmlLayout {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        serde_xml_rs::to_writer(buffer, self).map_err(From::from)
    }
}

impl From<XmlLayout> for UIC {
    fn from(value: XmlLayout) -> Self {
        let mut uic = Self::default();
        uic.source_is_xml = true;
        uic.version = value.version;
        uic.comment = value.comment;
        uic.precache_condition = value.precache_condition;
        uic
    }
}
