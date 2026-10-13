//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! XML forms of asset variation databases.
//!
//! The XML form in the packs lists the values of a category as repeated `value` attributes of one element, which
//! XML doesn't allow, so it's read with the small parser of this module instead of a standard one.

use super::*;

/// Categories of the Assembly Kit's environment map, in the order their bits are given.
const ENVIRONMENT_MAP_CATEGORIES: [&str; 8] = ["map", "climate", "season", "climate_change_index", "precipitation", "time_of_day", "battle_types", "snow_attrition"];

/// Key of the entry the environment map becomes.
const ENVIRONMENT_KEY: &str = "environment";

/// An element of an XML document, with its attributes in file order.
struct XmlElement {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<usize>,
}

//---------------------------------------------------------------------------//
//                    Implementation of AssetVariationDb
//---------------------------------------------------------------------------//

impl AssetVariationDb {

    /// Returns the database in the XML form of the packs: UTF-8, CRLF line ends, tab indentation and single-quoted attributes.
    pub fn to_xml(&self) -> String {
        let mut xml = String::from("<warscape_asset_variation_db serialise_version='1'>\r\n\t<meta_data_entries>\r\n");
        for category in &self.categories {
            xml.push_str(&format!("\t\t<meta_data_entry serialise_version='1' name='{}'>\r\n\t\t\t<values", escape(&category.name)));
            for value in &category.values {
                xml.push_str(&format!(" value='{}'", escape(value)));
            }
            xml.push_str("/>\r\n\t\t</meta_data_entry>\r\n");
        }

        xml.push_str("\t</meta_data_entries>\r\n\t<entries>\r\n");
        for entry in &self.entries {
            xml.push_str(&format!("\t\t<entry key='{}'>\r\n\t\t\t<value serialise_version='1'>\r\n\t\t\t\t<variations>\r\n", escape(&entry.key)));
            for variation in &entry.variations {
                xml.push_str(&format!("\t\t\t\t\t<variation serialise_version='1' filename='{}' flags='{}'/>\r\n", escape(&variation.filename), variation.flags));
            }
            xml.push_str("\t\t\t\t</variations>\r\n\t\t\t</value>\r\n\t\t</entry>\r\n");
        }

        xml.push_str("\t</entries>\r\n</warscape_asset_variation_db>\r\n");
        xml
    }

    /// Reads a database from the XML form of the packs. Whitespace between elements doesn't matter.
    ///
    /// # Errors
    ///
    /// Returns [`RLibError::DecodingAssetVariationDbXml`] if the XML is malformed, or doesn't have the structure of the database.
    pub fn from_xml(xml: &str) -> Result<Self> {
        let (elements, roots) = parse_elements(xml)?;
        let [root] = roots.as_slice() else { return Err(xml_error("there must be one root element".to_owned())) };
        let root = &elements[*root];
        if root.name != "warscape_asset_variation_db" {
            return Err(xml_error("the root element is not warscape_asset_variation_db".to_owned()));
        }

        check_version(root)?;

        let mut db = Self {
            serialise_version: 1,
            ..Default::default()
        };

        for &index in &child(&elements, root, "meta_data_entries")?.children {
            let element = &elements[index];
            if element.name != "meta_data_entry" {
                return Err(xml_error(format!("unexpected <{}> in meta_data_entries", element.name)));
            }

            check_version(element)?;
            let values = child(&elements, element, "values")?;
            let values = values.attributes.iter().map(|(name, value)| if name == "value" {
                Ok(value.to_owned())
            } else {
                Err(xml_error(format!("<values> attribute '{name}' is not 'value'")))
            }).collect::<Result<_>>()?;

            db.categories.push(VariationCategory {
                name: attribute(element, "name")?.to_owned(),
                values,
            });
        }

        for &index in &child(&elements, root, "entries")?.children {
            let element = &elements[index];
            if element.name != "entry" {
                return Err(xml_error(format!("unexpected <{}> in entries", element.name)));
            }

            let value = child(&elements, element, "value")?;
            check_version(value)?;

            let mut entry = VariationEntry {
                key: attribute(element, "key")?.to_owned(),
                variations: vec![],
            };

            for &index in &child(&elements, value, "variations")?.children {
                let variation = &elements[index];
                if variation.name != "variation" {
                    return Err(xml_error(format!("unexpected <{}> in variations", variation.name)));
                }

                check_version(variation)?;
                let filename = attribute(variation, "filename")?.to_owned();
                let flags = attribute(variation, "flags")?;

                // Only plain digits: the parser would also accept a sign.
                let flags = flags.parse::<u32>().ok().filter(|_| flags.bytes().all(|byte| byte.is_ascii_digit()))
                    .ok_or_else(|| xml_error(format!("flags '{flags}' of {filename} is not an unsigned 32-bit number")))?;

                entry.variations.push(Variation { filename, flags });
            }

            db.entries.push(entry);
        }

        Ok(db)
    }

    /// Builds a database from its source in the Assembly Kit, `raw_data/warscape_asset_variation_db/environment_map.xml`.
    ///
    /// Each `environment_file` lists the values it allows per category, comma-separated, and an empty or missing list allows every value.
    /// The database gets the single entry `environment`. The categories go in a fixed order, with their values sorted:
    /// the bit order the game's tools use isn't known, but it doesn't change what matches.
    ///
    /// # Errors
    ///
    /// Returns [`RLibError::DecodingAssetVariationDbXml`] if the XML is malformed, has no `environment_file`, or more values than bits.
    pub fn from_environment_map(xml: &str) -> Result<Self> {
        let (elements, _) = parse_elements(xml)?;
        let files = elements.iter().filter(|element| element.name == "environment_file").collect::<Vec<_>>();
        if files.is_empty() {
            return Err(xml_error("no <environment_file> entries".to_owned()));
        }

        let lists = files.iter().map(|file| {
            ENVIRONMENT_MAP_CATEGORIES.iter().map(|category| {
                file.attributes.iter()
                    .find(|(name, _)| name == category)
                    .map(|(_, list)| list.split(',').map(|value| value.trim_matches([' ', '\t'])).filter(|value| !value.is_empty()).collect::<Vec<_>>())
                    .unwrap_or_default()
            }).collect::<Vec<_>>()
        }).collect::<Vec<_>>();

        let mut db = Self {
            serialise_version: 1,
            ..Default::default()
        };

        for (index, name) in ENVIRONMENT_MAP_CATEGORIES.iter().enumerate() {
            let mut values = lists.iter().flat_map(|file| file[index].iter().map(|value| value.to_string())).collect::<Vec<_>>();
            values.sort();
            values.dedup();
            db.categories.push(VariationCategory { name: name.to_string(), values });
        }

        if db.value_count() > FLAG_BITS {
            return Err(xml_error(format!("more than {FLAG_BITS} category values, the flags would collide")));
        }

        let mut entry = VariationEntry {
            key: ENVIRONMENT_KEY.to_owned(),
            variations: Vec::with_capacity(files.len()),
        };

        for (file, lists) in files.iter().zip(&lists) {
            let allowed = ENVIRONMENT_MAP_CATEGORIES.iter().zip(lists)
                .filter(|(_, list)| !list.is_empty())
                .map(|(category, list)| (category.to_string(), list.iter().map(|value| value.to_string()).collect()))
                .collect();

            entry.variations.push(Variation {
                filename: file.attributes.iter().find(|(name, _)| name == "filename").map(|(_, value)| value.to_owned()).unwrap_or_default(),
                flags: db.flags_for(&allowed),
            });
        }

        db.entries.push(entry);
        Ok(db)
    }
}

/// Parses the elements of an XML document, keeping every attribute in order, repeated ones included.
///
/// Text, comments and declarations are skipped. Returns the elements and the indices of the root ones.
fn parse_elements(xml: &str) -> Result<(Vec<XmlElement>, Vec<usize>)> {
    let mut elements: Vec<XmlElement> = vec![];
    let mut roots = vec![];
    let mut open: Vec<usize> = vec![];
    let mut rest = xml;

    while let Some(start) = rest.find('<') {
        rest = &rest[start..];

        if let Some(skipped) = rest.strip_prefix("<?").map(|tail| ("?>", tail)).or_else(|| rest.strip_prefix("<!--").map(|tail| ("-->", tail))) {
            let end = skipped.1.find(skipped.0).ok_or_else(|| xml_error("unterminated declaration or comment".to_owned()))?;
            rest = &skipped.1[end + skipped.0.len()..];
            continue;
        }

        let end = tag_end(rest).ok_or_else(|| xml_error("unterminated tag".to_owned()))?;
        let tag = &rest[1..end];
        rest = &rest[end + 1..];

        if let Some(name) = tag.strip_prefix('/') {
            let closed = open.pop().ok_or_else(|| xml_error(format!("closing </{}> without an open element", name.trim())))?;
            if elements[closed].name != name.trim() {
                return Err(xml_error(format!("</{}> closes <{}>", name.trim(), elements[closed].name)));
            }
            continue;
        }

        let (tag, self_closing) = match tag.strip_suffix('/') {
            Some(tag) => (tag, true),
            None => (tag, false),
        };

        let name_end = tag.find(|char: char| char.is_whitespace()).unwrap_or(tag.len());
        let element = XmlElement {
            name: tag[..name_end].to_owned(),
            attributes: parse_attributes(&tag[name_end..])?,
            children: vec![],
        };

        let index = elements.len();
        match open.last() {
            Some(&parent) => elements[parent].children.push(index),
            None => roots.push(index),
        }

        elements.push(element);
        if !self_closing {
            open.push(index);
        }
    }

    if let Some(&unclosed) = open.last() {
        return Err(xml_error(format!("<{}> is not closed", elements[unclosed].name)));
    }

    Ok((elements, roots))
}

/// Returns the position of the `>` that ends the tag at the start of `text`, skipping the ones inside quoted values.
fn tag_end(text: &str) -> Option<usize> {
    let mut quote = None;
    for (index, char) in text.char_indices() {
        match (quote, char) {
            (None, '>') => return Some(index),
            (None, '\'' | '"') => quote = Some(char),
            (Some(open), _) if open == char => quote = None,
            _ => {}
        }
    }

    None
}

/// Parses the attributes of a tag, after its name.
fn parse_attributes(mut text: &str) -> Result<Vec<(String, String)>> {
    let mut attributes = vec![];
    loop {
        text = text.trim_start();
        if text.is_empty() {
            return Ok(attributes);
        }

        let equals = text.find('=').ok_or_else(|| xml_error(format!("attribute without value in '{text}'")))?;
        let name = text[..equals].trim().to_owned();
        let value = text[equals + 1..].trim_start();
        let quote = value.chars().next().filter(|char| *char == '\'' || *char == '"').ok_or_else(|| xml_error(format!("unquoted value of attribute '{name}'")))?;
        let end = value[1..].find(quote).ok_or_else(|| xml_error(format!("unterminated value of attribute '{name}'")))?;

        attributes.push((name, unescape(&value[1..end + 1])));
        text = &value[end + 2..];
    }
}

/// Returns the only child of an element with the provided name.
fn child<'a>(elements: &'a [XmlElement], parent: &XmlElement, name: &str) -> Result<&'a XmlElement> {
    let mut found = parent.children.iter().map(|&index| &elements[index]).filter(|element| element.name == name);
    match (found.next(), found.next()) {
        (Some(element), None) => Ok(element),
        (Some(_), Some(_)) => Err(xml_error(format!("more than one <{name}> in <{}>", parent.name))),
        (None, _) => Err(xml_error(format!("missing <{name}> in <{}>", parent.name))),
    }
}

/// Returns the value of an attribute of an element.
fn attribute<'a>(element: &'a XmlElement, name: &str) -> Result<&'a str> {
    element.attributes.iter()
        .find(|(attribute, _)| attribute == name)
        .map(|(_, value)| value.as_str())
        .ok_or_else(|| xml_error(format!("<{}> has no {name} attribute", element.name)))
}

/// Fails if the `serialise_version` of an element isn't 1.
fn check_version(element: &XmlElement) -> Result<()> {
    if attribute(element, "serialise_version")? != "1" {
        return Err(xml_error(format!("<{}> serialise_version is not 1", element.name)));
    }

    Ok(())
}

/// Escapes a value for a single-quoted attribute.
fn escape(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('\'', "&apos;")
}

/// Replaces the predefined entities of XML. Others are left as they are.
fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(start) = rest.find('&') {
        out.push_str(&rest[..start]);
        rest = &rest[start..];

        let replacement = rest.find(';').and_then(|end| match &rest[1..end] {
            "amp" => Some(('&', end)),
            "lt" => Some(('<', end)),
            "gt" => Some(('>', end)),
            "quot" => Some(('"', end)),
            "apos" => Some(('\'', end)),
            _ => None,
        });

        match replacement {
            Some((char, end)) => {
                out.push(char);
                rest = &rest[end + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }

    out.push_str(rest);
    out
}

/// Error of an XML database that can't be read.
fn xml_error(reason: String) -> RLibError {
    RLibError::DecodingAssetVariationDbXml(reason)
}
