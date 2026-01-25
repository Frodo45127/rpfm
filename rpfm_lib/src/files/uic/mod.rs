//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! UI Component (UIC) files for Total War games.
//!
//! UIC files define the layout, appearance, and behaviour of UI elements in the game.
//! They contain hierarchical component definitions with states, images, animations,
//! and callback bindings.
//!
//! # Format History
//!
//! - **Pre-Three Kingdoms**: Binary format (not yet implemented)
//! - **Three Kingdoms onwards**: XML format (partially implemented)
//!
//! # Status
//!
//! **This module is incomplete and experimental.** Only XML parsing for version 138
//! is partially implemented. Binary format support is not yet available.

use serde_derive::{Serialize, Deserialize};

use std::collections::HashMap;
use std::io::SeekFrom;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};

/// Path where all uics are in.
pub const BASE_PATH: &str = "ui";

/// Extension of UIC files in some games (they don't have extensions in some games).
pub const EXTENSIONS: [&str; 2] = [".cml", ".xml"];

mod xml;

//#[cfg(test)] mod uic_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// In-memory representation of a decoded UI Component file.
///
/// Contains the complete UI component definition including its hierarchy,
/// component definitions, and metadata.
///
/// # Fields
///
/// * `version` - Format version number.
/// * `source_is_xml` - `true` if decoded from XML format, `false` if binary.
/// * `comment` - Optional comment/description for the component.
/// * `precache_condition` - Condition for precaching this component.
/// * `hierarchy` - Tree structure of UI element relationships.
/// * `components` - Map of component IDs to their definitions.
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct UIC {
    version: u32,
    source_is_xml: bool,
    comment: String,
    precache_condition: String,
    hierarchy: HashMap<String, HierarchyItem>,
    components: HashMap<String, Component>,
}

/// A node in the UI component hierarchy tree.
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct HierarchyItem {
    /// Unique identifier for this hierarchy node.
    this: String,
    /// Child nodes in the hierarchy.
    childs: HashMap<String, HierarchyItem>,
}

/// A UI component definition with all its properties and sub-elements.
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Component {
    /// Unique identifier for this component.
    this: String,
    /// Human-readable component ID.
    id: String,
    /// Whether horizontal resizing is allowed.
    allowhorizontalresize: Option<bool>,
    /// Rendering priority.
    priority: Option<u8>,
    /// Whether tooltips should be localised.
    tooltipslocalised: Option<bool>,
    /// Globally unique identifier (typically same as `this`).
    uniqueguid: String,
    /// Whether to update this component when not visible.
    update_when_not_visible: Option<bool>,
    /// GUID of the current visual state.
    current_state: Option<String>,
    /// GUID of the default visual state.
    default_state: Option<String>,

    /// Event callbacks with context bindings.
    callbackwithcontextlist: Option<HashMap<String, CallbackWithContext>>,
    /// Image resources used by this component.
    componentimages: Option<HashMap<String, ComponentImage>>,
    /// Visual states (e.g., normal, hover, pressed).
    states: Option<HashMap<String, State>>,
    /// Animation definitions.
    animations: Option<HashMap<String, Animation>>,
    /// Layout engine configuration.
    layout_engine: Option<LayoutEngine>,
}

/// An event callback binding with optional context.
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct CallbackWithContext {
    /// Identifier of the callback function.
    callback_id: String,
    /// Optional context object for the callback.
    context_object_id: Option<String>,
    /// Optional context function for the callback.
    context_function_id: Option<String>,
    /// User-defined properties for this callback.
    child_m_user_properties: Option<HashMap<String, Property>>,
}

/// A key-value property pair.
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Property {
    /// Property name.
    name: String,
    /// Property value.
    value: String,
}

/// An image resource reference for a UI component.
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct ComponentImage {
    /// Unique identifier for this image reference.
    this: String,
    /// Globally unique identifier.
    uniqueguid: String,
    /// Path to the image file.
    imagepath: Option<String>,
}

/// A visual state definition for a UI component.
///
/// States define how a component appears under different conditions
/// (e.g., normal, hovered, pressed, disabled).
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    /// Unique identifier for this state.
    this: String,
    /// Human-readable state name.
    name: String,
    /// Component width in this state.
    width: Option<u32>,
    /// Text content to display.
    text: Option<String>,
    /// Horizontal text alignment.
    text_h_align: Option<String>,
    /// Vertical text offset.
    text_y_offset: Option<String>,
    /// Horizontal text behaviour.
    text_h_behaviour: Option<String>,
    /// Whether the text should be localised.
    text_localised: Option<bool>,
    /// Localisation key for the text.
    text_label: Option<String>,
    /// Font family name.
    font_m_font_name: Option<String>,
    /// Font size.
    font_m_size: Option<u8>,
    /// Font colour (hex format).
    font_m_colour: Option<String>,
    /// Line spacing (leading).
    font_m_leading: Option<u8>,
    /// Font category name.
    fontcat_name: Option<String>,
    /// Whether this state is interactive.
    interactive: Option<bool>,
    /// Globally unique identifier (typically same as `this`).
    uniqueguid: String,
    /// Image metrics for this state.
    imagemetrics: Option<HashMap<String, Image>>,
}

/// Image display properties within a UI state.
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Image {
    /// Unique identifier for this image instance.
    this: String,
    /// Globally unique identifier.
    uniqueguid: String,
    /// Reference to a ComponentImage.
    componentimage: String,
    /// Position offset.
    offset: Option<String>,
    /// Colour tint (hex format).
    colour: Option<String>,
    /// Docking anchor point.
    dockpoint: Option<String>,
    /// Offset from dock point.
    dock_offset: Option<String>,
    /// Whether height can be resized.
    canresizeheight: Option<bool>,
    /// Whether width can be resized.
    canresizewidth: Option<bool>,
    /// Colour preset key reference.
    ui_colour_preset_type_key: Option<String>,
}

/// An animation definition with keyframes.
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Animation {
    /// Animation identifier.
    id: String,
    /// Keyframes in this animation.
    frames: Vec<Frame>,
}

/// A single keyframe in an animation.
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Frame {
    /// Time to interpolate to this frame (milliseconds).
    interpolationtime: Option<u32>,
    /// Bitmask of properties to interpolate.
    interpolationpropertymask: Option<u8>,
    /// Target height at this keyframe.
    targetmetrics_m_height: Option<i32>,
    /// Target width at this keyframe.
    targetmetrics_m_width: Option<i32>
}

/// Layout engine configuration for arranging child components.
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct LayoutEngine {
    /// Layout type (e.g., "horizontal", "vertical", "grid").
    r#type: String,
    /// Spacing between child elements.
    spacing: String,
    /// Whether to resize to fit content.
    sizetocontent: bool,
    /// Margin values.
    margins: String,
    /// Fixed column widths for grid layouts.
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
