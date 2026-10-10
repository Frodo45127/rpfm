//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use super::*;

#[test]
fn test_sizes_in_the_buffer_are_minimal() {
    let mut stream = EsfStream::new(16);
    stream.write_sized(&[1, 2, 3]).unwrap();

    assert_eq!(stream.into_inner(), vec![3, 1, 2, 3]);
}

#[test]
fn test_sizes_open_at_a_flush_keep_their_placeholder() {
    let mut stream = EsfStream::new(16);
    let field = stream.open_size().unwrap();
    let big = vec![7; BUFFER_SIZE];
    stream.write_all(&big).unwrap();
    let size = stream.len() - stream.end_of(&field);
    let end = stream.close_size(field, size as u32).unwrap();

    // 0x10_0000 in CAULEB128 is 0xC0 0x80 0x00, padded to 5 bytes with leading 0x80.
    let data = stream.into_inner();
    assert_eq!(end, PLACEHOLDER_SIZE);
    assert_eq!(&data[..PLACEHOLDER_SIZE], &[0x80, 0x80, 0xC0, 0x80, 0x00]);
}

#[test]
fn test_sizes_closed_after_the_buffer_fills_keep_their_placeholder() {
    let mut stream = EsfStream::new(16);
    let outer = stream.open_size().unwrap();
    stream.write_sized(&[1, 2]).unwrap();
    let filler = vec![0; BUFFER_SIZE - 16 - PLACEHOLDER_SIZE - 3];
    stream.write_all(&filler).unwrap();
    let inner = stream.open_size().unwrap();
    stream.write_all(&[9]).unwrap();
    stream.close_size(inner, 1).unwrap();
    let size = stream.len() - stream.end_of(&outer);
    stream.close_size(outer, size as u32).unwrap();

    // The inner size was written after the flush, so it shrinks; the outer one was open across it.
    let data = stream.into_inner();
    assert_eq!(&data[..2], &[0x80, 0x80]);
    assert_eq!(&data[data.len() - 2..], &[1, 9]);
}
