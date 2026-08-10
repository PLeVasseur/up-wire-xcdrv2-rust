/********************************************************************************
 * Copyright (c) 2026 Contributors to the Eclipse Foundation
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

use up_rust::{DecodePayload, EncodePayload, ReadDecodePayload, UWireReadDecode};
use up_wire_xcdrv2::{VehicleSignalV1, XcdrV2Wire, VEHICLE_SIGNAL_V1_GOLDEN_VALUE};
use up_wire_xcdrv2::VEHICLE_SIGNAL_V1_DECODE_LIMIT;

fn assert_reader_decode<W>()
where
    W: UWireReadDecode<VehicleSignalV1>,
{
}

fn main() {
    assert_reader_decode::<XcdrV2Wire>();

    let encoded = XcdrV2Wire::encode_payload_owned(&VEHICLE_SIGNAL_V1_GOLDEN_VALUE)
        .expect("encode supported XCDRv2 fixture");
    let decoded: VehicleSignalV1 =
        XcdrV2Wire::decode_payload(&encoded).expect("decode supported XCDRv2 fixture");
    assert_eq!(decoded, VEHICLE_SIGNAL_V1_GOLDEN_VALUE);
    let decoded_from_reader: VehicleSignalV1 = XcdrV2Wire::decode_payload_from_reader(
        std::io::Cursor::new(&encoded),
        encoded.len(),
        VEHICLE_SIGNAL_V1_DECODE_LIMIT,
    )
    .expect("reader decode supported XCDRv2 fixture");
    assert_eq!(decoded_from_reader, VEHICLE_SIGNAL_V1_GOLDEN_VALUE);
}
