//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for decoding/encoding `TradeRoutes` files.

use std::io::Cursor;

use crate::files::*;

use super::*;

/// Builds a small trade routes file: two regions, one 1-segment spline, one region route and one port route.
fn sample_file() -> Vec<u8> {
    let mut data = SIGNATURE.to_vec();
    data.extend(1u32.to_le_bytes());

    data.extend(2u32.to_le_bytes());
    for name in ["reg_a", "reg_b"] {
        data.extend((name.len() as u32).to_le_bytes());
        data.extend(name.as_bytes());
    }

    data.extend(1u32.to_le_bytes());
    data.extend([1, 0, 1, 0]);
    data.extend(1u32.to_le_bytes());
    for value in [4336u16, 432] { data.extend(value.to_le_bytes()); }
    for value in [0i16, 4320, 0, -4320] { data.extend(value.to_le_bytes()); }
    for value in [4336u16, 576] { data.extend(value.to_le_bytes()); }

    for (from, to, edge) in [(0u32, 1u32, 0u32), (1, 0, 1)] {
        data.extend(1u32.to_le_bytes());
        data.extend(from.to_le_bytes());
        data.extend(to.to_le_bytes());
        data.extend(1u32.to_le_bytes());
        data.extend(edge.to_le_bytes());
    }

    data.extend(0xDEADBEEFu32.to_le_bytes());
    data
}

#[test]
fn test_decode_trade_routes() {
    let decoded = TradeRoutes::decode(&mut Cursor::new(sample_file()), &None).unwrap();

    assert_eq!(decoded.regions(), &vec!["reg_a".to_owned(), "reg_b".to_owned()]);
    assert_eq!(decoded.splines()[0].flags(), &[true, false, true, false]);
    assert_eq!(decoded.splines()[0].segments()[0].start(), &(4336, 432));
    assert_eq!(decoded.splines()[0].segments()[0].end_control(), &(0, -4320));
    assert_eq!(decoded.region_routes()[0].edges(), &vec![TradeRouteEdge::new(0)]);
    assert!(decoded.port_routes()[0].edges()[0].is_reversed());
    assert_eq!(*decoded.trailer(), 0xDEADBEEF);
}

#[test]
fn test_decode_trade_routes_rejects_bad_signature() {
    let mut data = sample_file();
    data[1] = b'X';

    assert!(matches!(TradeRoutes::decode(&mut Cursor::new(data), &None), Err(RLibError::DecodingPTDUnsupportedSignature(_))));
}

#[test]
fn test_encode_trade_routes() {
    let before = sample_file();
    let mut decoded = TradeRoutes::decode(&mut Cursor::new(before.clone()), &None).unwrap();

    let mut after = vec![];
    decoded.encode(&mut after, &None).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_trade_route_edge() {
    let edge = TradeRouteEdge::new(3217);

    assert_eq!(edge.spline(), 1608);
    assert!(edge.is_reversed());
}
