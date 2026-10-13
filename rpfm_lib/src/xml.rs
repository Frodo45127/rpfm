//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Minimal XML reading for formats that need more than serde: attributes in file order, repeated ones included.
//!
//! Text, comments and declarations are skipped. Errors are plain descriptions, which each format wraps in its own error.

/// Result of the XML helpers: errors are descriptions of what's wrong.
pub(crate) type XmlResult<T> = std::result::Result<T, String>;

/// An element of an XML document, with its attributes in file order.
pub(crate) struct XmlElement {
    pub(crate) name: String,
    pub(crate) attributes: Vec<(String, String)>,
    pub(crate) children: Vec<usize>,
}

/// Parses the elements of an XML document, keeping every attribute in order, repeated ones included.
///
/// Text, comments and declarations are skipped. Returns the elements and the indices of the root ones.
pub(crate) fn parse_elements(xml: &str) -> XmlResult<(Vec<XmlElement>, Vec<usize>)> {
    let mut elements: Vec<XmlElement> = vec![];
    let mut roots = vec![];
    let mut open: Vec<usize> = vec![];
    let mut rest = xml;

    while let Some(start) = rest.find('<') {
        rest = &rest[start..];

        if let Some(skipped) = rest.strip_prefix("<?").map(|tail| ("?>", tail)).or_else(|| rest.strip_prefix("<!--").map(|tail| ("-->", tail))) {
            let end = skipped.1.find(skipped.0).ok_or_else(|| "unterminated declaration or comment".to_owned())?;
            rest = &skipped.1[end + skipped.0.len()..];
            continue;
        }

        let end = tag_end(rest).ok_or_else(|| "unterminated tag".to_owned())?;
        let tag = &rest[1..end];
        rest = &rest[end + 1..];

        if let Some(name) = tag.strip_prefix('/') {
            let closed = open.pop().ok_or_else(|| format!("closing </{}> without an open element", name.trim()))?;
            if elements[closed].name != name.trim() {
                return Err(format!("</{}> closes <{}>", name.trim(), elements[closed].name));
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
        return Err(format!("<{}> is not closed", elements[unclosed].name));
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
fn parse_attributes(mut text: &str) -> XmlResult<Vec<(String, String)>> {
    let mut attributes = vec![];
    loop {
        text = text.trim_start();
        if text.is_empty() {
            return Ok(attributes);
        }

        let equals = text.find('=').ok_or_else(|| format!("attribute without value in '{text}'"))?;
        let name = text[..equals].trim().to_owned();
        let value = text[equals + 1..].trim_start();
        let quote = value.chars().next().filter(|char| *char == '\'' || *char == '"').ok_or_else(|| format!("unquoted value of attribute '{name}'"))?;
        let end = value[1..].find(quote).ok_or_else(|| format!("unterminated value of attribute '{name}'"))?;

        attributes.push((name, unescape(&value[1..end + 1])));
        text = &value[end + 2..];
    }
}

/// Replaces the predefined entities of XML. Others are left as they are.
pub(crate) fn unescape(value: &str) -> String {
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
