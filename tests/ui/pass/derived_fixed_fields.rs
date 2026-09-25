/********************************************************************************
 * Copyright (c) 2026 Contributors to the Eclipse Foundation
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

use up_rust::{DecodePayload, PayloadDecodeLimit, ReadDecodePayload};
use up_wire_xcdrv2::{XcdrV2Mappable, XcdrV2Payload, XcdrV2Type, XcdrV2Wire};

#[derive(Clone, Debug, PartialEq, XcdrV2Type)]
#[xcdr_v2(type_name = "tests.FixedPayload")]
struct FixedPayload {
    sequence: u32,
    enabled: bool,
    samples: [i32; 3],
    checksum: u64,
}

fn main() {
    let value = FixedPayload {
        sequence: 42,
        enabled: true,
        samples: [-7, 0, 99],
        checksum: 0x0123_4567_89ab_cdef,
    };
    let encoded = XcdrV2Payload::encode(&value).expect("encode fixed fields");
    assert_eq!(encoded.as_bytes().len(), FixedPayload::ENCODED_LEN);

    let decoded: FixedPayload =
        XcdrV2Wire::decode_payload(encoded.as_bytes()).expect("decode fixed fields");
    assert_eq!(decoded, value);

    let decoded_from_reader: FixedPayload = XcdrV2Wire::decode_payload_from_reader(
        std::io::Cursor::new(encoded.as_bytes()),
        encoded.as_bytes().len(),
        PayloadDecodeLimit::new(FixedPayload::ENCODED_LEN),
    )
    .expect("reader decode fixed fields");
    assert_eq!(decoded_from_reader, value);
}
