//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `Cs2Parsed` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

//use cs2_collision::Cs2Collision;
//use cs2_parsed::Transform4x4;

use crate::binary::ReadBytes;
use crate::files::*;

use super::Cs2Parsed;

#[test]
fn test_encode_cs2_parsed() {
    let path_1 = "../test_files/test_wall.cs2.parsed";
    let path_2 = "../test_files/test_encode_wall.cs2.parsed";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Cs2Parsed::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];

    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_cs2_corner_parsed() {
    let path_1 = "../test_files/test_wall_corner.cs2.parsed";
    let path_2 = "../test_files/test_encode_wall_corner.cs2.parsed";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Cs2Parsed::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];

    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_cs2_parsed_v20_3k() {
    let path_1 = "../test_files/test_decode_v20_3k.cs2.parsed";
    let path_2 = "../test_files/test_encode_v20_3k.cs2.parsed";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Cs2Parsed::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];

    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_cs2_parsed_v21() {
    let path_1 = "../test_files/test_decode_v21.cs2.parsed";
    let path_2 = "../test_files/test_encode_v21.cs2.parsed";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Cs2Parsed::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];

    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

/*
#[test]
fn test_assladers_begone() {
    let remove_assladders = true;
    let patch_walls = false;

    let base_path = "C:/Users/frodo/Desktop/assladers/rigidmodels/buildings";
    let patched_platforms_path = "C:/Users/frodo/Desktop/assladers/rigidmodels/buildings_prepatched_platforms";
    let dest_path = PathBuf::from("C:/Users/frodo/Desktop/assladers/rigidmodels/buildings_fixed");
    let paths = files_from_subdir(&PathBuf::from(base_path), true).unwrap();
    let prepatched_paths = files_from_subdir(&PathBuf::from(patched_platforms_path), true).unwrap();

    for path in &paths {
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        if file_name.ends_with(".cs2.parsed") {
            println!("Checking file: {}", path.to_string_lossy().to_string().replace("\\", "/"));

            let mut reader = BufReader::new(File::open(&path).unwrap());

            let decodeable_extra_data = DecodeableExtraData::default();

            let data_len = reader.len().unwrap();
            let before = reader.read_slice(data_len as usize, true).unwrap();
            match Cs2Parsed::decode(&mut reader, &Some(decodeable_extra_data.clone())) {
                Ok(mut data) => {
                    let mut after = vec![];

                    if remove_assladders {

                        // Remove ladder pipes.
                        for piece in &mut data.pieces {
                            for destruct in &mut piece.destructs {
                                dbg!(&destruct.pipes);
                                destruct.pipes.retain(|pipe| !pipe.name.contains("ladder"));
                            }
                        }
                    }

                    if patch_walls {

                        // Fixes for the wall pillars. These should be applied to any wall piece of the following type:
                        // - straigh_20m
                        if file_name.contains("_wall_straight_20m_") {

                            if let Some(piece) = data.pieces.first_mut() {
                                if let Some(destruct) = piece.destructs.last_mut() {

                                    // Add left and right model data only to the last destruct of piece01.
                                    if destruct.file_refs.is_empty() {
                                        let mut file_name_reduced = file_name.to_owned();
                                        file_name_reduced.truncate(file_name.find(".").unwrap());

                                        let mut right = FileRef::default();
                                        right.name = format!("{}_bridging_column_right", file_name_reduced);
                                        right.key = format!("{}_file:{}", piece.name, right.name);
                                        right.uk_1 = 19;
                                        right.transform = Transform4x4::identity();
                                        destruct.file_refs.push(right);

                                        let mut left = FileRef::default();
                                        left.name = format!("{}_bridging_column_left", file_name_reduced);
                                        left.key = format!("{}_file:{}", piece.name, left.name);
                                        left.uk_1 = 20;
                                        left.transform = Transform4x4::identity();
                                        destruct.file_refs.push(left);
                                    }

                                    // Remove all collisions.
                                    destruct.collision_outlines.clear();
                                    destruct.orange_thingies.clear();

                                    // Check if we have prepatched platforms for it, and use them instead of their own.
                                    let patched_path_end = path.strip_prefix(base_path).unwrap();
                                    let patched_path = PathBuf::from(patched_platforms_path).join(patched_path_end);

                                    // TODO: USE GLB instead of faction specific as a fallback.
                                    if let Some(patched_path) = prepatched_paths.iter().find(|x| **x == patched_path) {
                                        let mut reader = BufReader::new(File::open(&patched_path).unwrap());
                                        if let Ok(data) = Cs2Parsed::decode(&mut reader, &Some(decodeable_extra_data)) {
                                            if let Some(destruct_patched) = data.pieces.iter().find_map(|x| x.destructs.iter().find(|x| x.name == destruct.name)) {
                                                println!("Platforms patched with prepatched file: {}", patched_path.to_string_lossy());
                                                destruct.platforms.clear();
                                                destruct.platforms = destruct_patched.platforms.to_vec();
                                            }
                                        }
                                    }

                                    /*
                                    // Add new platforms to make the new collision-less sides walkable.
                                    // The best way I have been able to come up with to do it:
                                    // - Find the biggest vertex to start calculating the perimeter.
                                    // - Find all perimeter vertex via the "look back and find the first line" trick.
                                    // - Go through the vertex, and split them in concave areas.
                                    // - Merge said areas with the border of the 20x20 cube.
                                    if let Some((_, starting_point)) = destruct.platforms.iter()
                                        .map(|x| {
                                            let outline = x.vertices.outline();
                                            outline.iter()
                                                .map(|x| (((x.x() + x.z()) / 2.0), x))
                                                .max_by(|x, y| x.0.total_cmp(&y.0)).unwrap()
                                        })
                                        .max_by(|x, y| {
                                            x.0.total_cmp(&y.0)
                                        }) {


                                        let mut points = vec![starting_point.clone()];
                                        loop {
                                            let pre_vertex = if let Some(ver) = points.get((points.len() as i32 - 2) as usize) { ver.clone() } else { Point3d::new(20.0, 0.0, 6.0) };
                                            let current_vertex = points.last().unwrap();
                                            match destruct.platforms.iter()
                                                .map(|x| x.vertices.outline())
                                                .filter(|x| x.contains(&current_vertex))   // Get only the platforms with our vertex, because we are searching one that has an external edge.
                                                .map(|x| x.iter()
                                                    .filter(|y| !points.contains(y))
                                                    .filter_map(|y| {

                                                        // We need to get the angle between the pre-current and current-y vectors.
                                                        let vec_1 = pre_vertex - *current_vertex;
                                                        let vec_2 = *y - *current_vertex;
                                                        let mut angle = vec_1.z().atan2(*vec_1.x()) - vec_2.z().atan2(*vec_2.x());
                                                        if angle < 0.0 {
                                                            angle += 2.0 * PI;
                                                        }
                                                        if angle > 0.0 {
                                                            Some((angle, y))
                                                        } else {
                                                            None
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()
                                                )
                                                .flatten()
                                                .max_by(|x, y| x.0.total_cmp(&y.0)) {
                                                Some((_, next_vertex)) => points.push(next_vertex.clone()),
                                                None => break,
                                            }
                                        }

                                        // TODO: calculate this based on the geometry of the model.
                                        let border_corners_and_mid_points = vec![
                                            Point3d::new(0.0, 0.0, -14.0),
                                            Point3d::new(20.0, 0.0, -14.0),
                                            Point3d::new(20.0, 0.0, 6.0),
                                            Point3d::new(0.0, 0.0, 6.0),
                                            //Point3d::new(0.0, 10.0, 0.0),
                                            //Point3d::new(20.0, 10.0, 0.0),
                                            //Point3d::new(10.0, 0.0, 0.0),
                                            //Point3d::new(10.0, 20.0, 0.0),
                                        ];

                                        // If everything worked as expected, we will have all the points that form the perimeter of the platforms, in sequence.
                                        // Now we need to generate new platforms using the closest two corners of the bounding box and their edge's center as third vertex.
                                        let point_default = Point3d::default();
                                        let mut prev_closest_point = Point3d::default();
                                        for i in 0..points.len() {
                                            let j = if i as i32 == points.len() as i32 - 1 { 0 } else { i + 1 };
                                            let current = &points[i];
                                            let next = &points[j];

                                            // To get to which corner or mid edge we need to join, we need to get the closest to the mid point of the edge between the vertices.
                                            // Fucking maths.
                                            let mid = ((current.x() + next.x()) / 2.0, (current.z() + next.z()) / 2.0);
                                            let (closest_point, _) = border_corners_and_mid_points.par_iter()
                                                .map(|x| (x, (x.x() - mid.0, x.z() - mid.1)))
                                                .map(|x| (x.0, (x.1.0.powi(2) + x.1.1.powi(2)).sqrt()))
                                                .min_by(|x, y| x.1.total_cmp(&y.1))
                                                .unwrap();

                                            // Fix for the first change in closest point.
                                            if prev_closest_point == point_default {
                                                prev_closest_point = closest_point.clone();
                                            }

                                            // If we're changing the closest point, we need to also generate the platform between the edges and the first vertex of our pair.
                                            if prev_closest_point != *closest_point {
                                                let mut platform = Platform::default();
                                                platform.flag_2 = true;
                                                platform.vertices.set_outline(vec![
                                                    closest_point.clone(),
                                                    prev_closest_point.clone(),
                                                    current.clone()
                                                ]);

                                                platform.normal = {
                                                    let a = platform.vertices.outline()[1] - platform.vertices.outline()[0];
                                                    let b = platform.vertices.outline()[2] - platform.vertices.outline()[0];
                                                    Point3d::new(
                                                        a.y() * b.z() - a.z() * b.y(),
                                                        a.z() * b.x() - a.x() * b.z(),
                                                        a.x() * b.y() - a.y() * b.x()
                                                    )
                                                };

                                                destruct.platforms.push(platform);

                                                prev_closest_point = closest_point.clone();
                                            }

                                            // Generate the platform.
                                            let mut platform = Platform::default();
                                            platform.flag_2 = true;
                                            platform.vertices.set_outline(vec![
                                                prev_closest_point.clone(),
                                                current.clone(),
                                                next.clone(),
                                            ]);

                                            platform.normal = {
                                                let a = platform.vertices.outline()[1] - platform.vertices.outline()[0];
                                                let b = platform.vertices.outline()[2] - platform.vertices.outline()[0];
                                                Point3d::new(
                                                    a.y() * b.z() - a.z() * b.y(),
                                                    a.z() * b.x() - a.x() * b.z(),
                                                    a.x() * b.y() - a.y() * b.x()
                                                )
                                            };

                                            destruct.platforms.push(platform);
                                        }
                                    }*/
                                }
                            }
                        }
                    }

                    data.encode(&mut after, &None).unwrap();

                    if before != after {
                        let dest_path_end = path.strip_prefix(base_path).unwrap();
                        let dest_path = dest_path.join(dest_path_end);
                        let mut dest_path_parent = dest_path.to_path_buf();
                        dest_path_parent.pop();

                        DirBuilder::new().recursive(true).create(&dest_path_parent).unwrap();

                        let mut writer = BufWriter::new(File::create(&dest_path).unwrap());
                        writer.write_all(&after).unwrap();

                        println!("File edited: {}", dest_path.to_string_lossy().to_string());
                    }
                }
                Err(error) => {
                    let dest_path_end = path.strip_prefix(base_path).unwrap();
                    let dest_path = dest_path.join(dest_path_end);

                    println!("Failed to read file: {}, with error: {}", dest_path.to_string_lossy().to_string(), error);

                }
            }
        }

        if path.file_name().unwrap().to_string_lossy().to_string().ends_with(".cs2.collision") {
            let mut reader = BufReader::new(File::open(&path).unwrap());

            let decodeable_extra_data = DecodeableExtraData::default();

            let data_len = reader.len().unwrap();
            let before = reader.read_slice(data_len as usize, true).unwrap();
            match Cs2Collision::decode(&mut reader, &Some(decodeable_extra_data)) {
                Ok(mut data) => {
                    let mut after = vec![];

                    if patch_walls {

                        // Fixes for the wall pillars. These should be applied to any wall piece of the following type:
                        // - straigh_20m
                        if file_name.contains("_wall_straight_20m_") {

                            if let Some(collision_3d) = data.collisions_3d_mut().last_mut() {

                                // Remove all collision data from the last collision set.
                                collision_3d.triangles_mut().clear();
                                collision_3d.vertices_mut().clear();
                            }
                        }
                    }

                    data.encode(&mut after, &None).unwrap();

                    if before != after {
                        let dest_path_end = path.strip_prefix(base_path).unwrap();
                        let dest_path = dest_path.join(dest_path_end);
                        let mut dest_path_parent = dest_path.to_path_buf();
                        dest_path_parent.pop();

                        DirBuilder::new().recursive(true).create(&dest_path_parent).unwrap();

                        let mut writer = BufWriter::new(File::create(&dest_path).unwrap());
                        writer.write_all(&after).unwrap();

                        println!("File edited: {}", dest_path.to_string_lossy().to_string());
                    }
                }
                Err(error) => {
                    let dest_path_end = path.strip_prefix(base_path).unwrap();
                    let dest_path = dest_path.join(dest_path_end);

                    println!("Failed to read file: {}, with error: {}", dest_path.to_string_lossy().to_string(), error);

                }
            }
        }
    }
}
*/
