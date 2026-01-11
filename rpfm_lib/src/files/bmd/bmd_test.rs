//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `Bmd` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use quick_xml::{events::Event, Reader, Writer};

use crate::binary::ReadBytes;
use crate::files::*;

use super::{Bmd, ToLayer};

#[test]
fn test_encode_bmd_prefab() {
    let path_1 = "../test_files/test_decode.bmd";
    let path_2 = "../test_files/test_encode.bmd";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Bmd::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_bmd_map_data_v23() {
    let path_1 = "../test_files/fastbin/v23_bmd_data.bin";
    let path_2 = "../test_files/fastbin/v23_encode_bmd_data.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Bmd::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_bmd_map_data_v23_old() {
    let path_1 = "../test_files/fastbin/v23_old_bmd_data.bin";
    let path_2 = "../test_files/fastbin/v23_encode_old_bmd_data.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Bmd::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_bmd_map_data_v24() {
    let path_1 = "../test_files/fastbin/v24_bmd_data.bin";
    let path_2 = "../test_files/fastbin/v24_encode_bmd_data.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Bmd::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_bmd_map_data_v27() {
    let path_1 = "../test_files/fastbin/v27_bmd_data.bin";
    let path_2 = "../test_files/fastbin/v27_encode_bmd_data.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Bmd::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_bmd_map_nogo_data() {
    let path_1 = "../test_files/fastbin/bmd_nogo_data.bin";
    let path_2 = "../test_files/fastbin/encode_bmd_nogo_data.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Bmd::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_bmd_map_catchment() {
    let path_1 = "../test_files/fastbin/catchment__bmd.bin";
    let path_2 = "../test_files/fastbin/encode_catchment__bmd.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Bmd::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_bmd_to_layer() {
    let path_1 = "../test_files/fastbin/prefabs/suerto_lzd_cliff_block.bmd";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();
    let data = Bmd::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    data.export_prefab_to_raw_data("test", None, &PathBuf::from("../test_files/fastbin/prefabs")).unwrap();
}

pub fn prettify_xml(xml: &str) -> String {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 4);


    loop {
        let ev = reader.read_event();

        match ev {
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Ok(event) => writer.write_event(event),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
            .expect("Failed to parse XML");
    }

    let result = std::str::from_utf8(&*writer.into_inner())
        .expect("Failed to convert a slice of bytes to a string slice")
        .to_string();

    result
}
#[test]
fn test_custom_klissan() {
    println!("");
    // let folder_path = "/media/user/990Plus/terrain/tiles/battle/multiplayer/schwartzhafen_mp/";
    let folder_path = "/media/user/990Plus/ffa/terrain/tiles/battle/domination/ffacapture/";
    let paths = files_from_subdir(&PathBuf::from(folder_path), true).unwrap();
    let mut failures = 0;
    // let mut heigh_modes = HashSet::new();
    let decodeable_extra_data = Some(DecodeableExtraData::default());
    for path in &paths {
        if path.extension().unwrap() == "bin" && path.file_name().unwrap().to_str().unwrap().contains("bmd_data") {
            println!("{:?}", path);
            let mut reader = BufReader::new(File::open(path).unwrap());
            match Bmd::decode(&mut reader, &decodeable_extra_data) {
                Ok(mut data) => {
                    let mut buffer = String::new();
                    dbg!(&data);
                    // buffer.push_str(&data.capture_location_set.to_layer(&data));
                    match data.capture_location_set.to_layer(&data) {
                        Ok(layer_string) => buffer.push_str(&layer_string),
                        Err(e) => panic!("Failed to convert to layer: {}", e),
                    }
/*                    match data.ai_hints.to_layer(&data) {
                        Ok(layer_string) => buffer.push_str(&layer_string),
                        Err(e) => panic!("Failed to convert to layer: {}", e),
                    }*/
/*                    match data.playable_area.to_layer(&data) {
                        Ok(layer_string) => buffer.push_str(&layer_string),
                        Err(e) => panic!("Failed to convert to layer: {}", e),
                    }*/
                    // println!("{}", buffer);
/*                    for building in data.battlefield_building_list().buildings() {
                        heigh_modes.insert(building.height_mode().to_owned());
                    }

                    for prop in data.prop_list().props() {
                        heigh_modes.insert(prop.height_mode().to_owned());
                    }*/
                    let mut reader_test = BufReader::new(File::open(path).unwrap());
                    let data_len = reader_test.len().unwrap();
                    let before = reader_test.read_slice(data_len as usize, true).unwrap();
                    let mut after = vec![];
                    data.encode(&mut after, &None).unwrap();
                    assert_eq!(before, after);

                    let xml_data = quick_xml::se::to_string(&data).expect("Failed to serialize data to XML");
                    let pretty_data = prettify_xml(&xml_data);
                    println!("{}", pretty_data);

                    let mut writer = BufWriter::new(File::create(folder_path.to_owned() + "pretty.xml").unwrap());
                    writer.write_all(pretty_data.as_bytes()).unwrap();

                    data.export_prefab_to_raw_data("export", None, &PathBuf::from("/media/user/990Plus/terrain/tiles/battle/multiplayer/schwartzhafen_mp/")).unwrap();
                },
                Err(error) => {
                    println!("\t{}:", error);
                    println!("\t\t - {}", path.to_string_lossy());
                    failures += 1;
                    //break;
                }
            }
        }
    }

    println!("Total errors: {}", failures);
    // dbg!(heigh_modes);
}

/*#[test]
fn test_mass_decode() {
    let folder_path = "/home/frodo45127/Proyectos/rpfm_test_files2/prefabs/";
    let paths = files_from_subdir(&PathBuf::from(folder_path), true).unwrap();
    let mut failures = 0;
    let mut heigh_modes = HashSet::new();
    let decodeable_extra_data = Some(DecodeableExtraData::default());
    for path in &paths {
        if path.extension().unwrap() == "bmd" {
            let mut reader = BufReader::new(File::open(path).unwrap());
            match Bmd::decode(&mut reader, &decodeable_extra_data) {
                Ok(data) => {
                    for building in data.battlefield_building_list().buildings() {
                        heigh_modes.insert(building.height_mode().to_owned());
                    }

                    for prop in data.prop_list().props() {
                        heigh_modes.insert(prop.height_mode().to_owned());
                    }
                },
                Err(error) => {
                    println!("\t{}:", error);
                    println!("\t\t - {}", path.to_string_lossy());
                    failures += 1;
                    //break;
                }
            }
        }
    }

    println!("Total errors: {}", failures);
    dbg!(heigh_modes);
}*/

