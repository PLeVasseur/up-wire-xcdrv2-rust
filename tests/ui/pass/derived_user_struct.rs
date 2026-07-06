/********************************************************************************
 * Copyright (c) 2026 Contributors to the Eclipse Foundation
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

use up_rust::{DecodePayload, EncodePayload, ReadDecodePayload};
use up_rust::wire_implementer_api::{UWirePayload, UWireReadDecode};
use up_wire_xcdrv2::{XcdrV2Payload, XcdrV2Type, XcdrV2Wire};

#[derive(Clone, Debug, PartialEq, XcdrV2Type)]
#[xcdr_v2(type_name = "tests.Nested")]
struct Nested {
    enabled: bool,
    value: i32,
}

#[derive(Clone, Debug, PartialEq, XcdrV2Type)]
#[xcdr_v2(type_name = "tests.UserPayload")]
struct UserPayload {
    id: u32,
    nested: Nested,
    fixed: [u16; 2],
    name: String,
    samples: Vec<i64>,
    maybe: Option<u8>,
}

fn assert_reader_decode<W>()
where
    W: UWireReadDecode<UserPayload>,
{
}

fn assert_selected_wire_payload<W>()
where
    W: UWirePayload<UserPayload>,
{
}

fn main() {
    assert_reader_decode::<XcdrV2Wire>();
    assert_selected_wire_payload::<XcdrV2Wire>();

    let value = UserPayload {
        id: 42,
        nested: Nested {
            enabled: true,
            value: -7,
        },
        fixed: [1, 2],
        name: "example".to_string(),
        samples: vec![3, 4, 5],
        maybe: Some(8),
    };
    let encoded = XcdrV2Payload::<UserPayload>::encode(&value).expect("encode");
    let decoded: UserPayload = XcdrV2Wire::decode_payload(encoded.as_bytes()).expect("decode");
    assert_eq!(decoded, value);

    let owned = XcdrV2Wire::encode_payload_owned(&value).expect("owned encode");
    let decoded_from_reader: UserPayload = XcdrV2Wire::decode_payload_from_reader(
        std::io::Cursor::new(&owned),
        owned.len(),
    )
    .expect("reader decode");
    assert_eq!(decoded_from_reader, value);
}
