/********************************************************************************
 * Copyright (c) 2026 Contributors to the Eclipse Foundation
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

use up_rust::{
    EncodePayload, NativePrefixFrameMetadataCodec, PayloadCodec, PreparedTxLoanSpec,
    ReadDecodePayload, UFrameMetadata, UTxBuffer, UTxLoanSpec, UUri, UVecTxBuffer, UWire,
    UWireMetadataCodec,
};
use up_wire_xcdrv2::{
    VehicleSignalV1, XcdrV2Mappable, XcdrV2Payload, XcdrV2Type, XcdrV2Wire,
    VEHICLE_SIGNAL_V1_GOLDEN_BYTES, VEHICLE_SIGNAL_V1_GOLDEN_VALUE,
};

static_assertions::assert_not_impl_any!(XcdrV2Wire: up_rust::BorrowPayload<VehicleSignalV1>);

#[derive(Clone, Debug, PartialEq, XcdrV2Type)]
#[xcdr_v2(type_name = "tests.ExternalFixedPayload")]
struct ExternalFixedPayload {
    sequence: u32,
    ready: bool,
    values: [i32; 3],
    checksum: u64,
}

#[test]
fn serialized_zero_copy_tx_fixture_writes_xcdrv2_bytes_into_loan() {
    let metadata = metadata();
    let layout = XcdrV2Wire::payload_layout(&VEHICLE_SIGNAL_V1_GOLDEN_VALUE)
        .expect("measure XCDRv2 payload");
    let spec = UTxLoanSpec::payload(metadata.clone(), layout.len(), layout.align())
        .expect("create TX loan spec");
    let codec = NativePrefixFrameMetadataCodec;
    let prepared =
        PreparedTxLoanSpec::from_validated::<XcdrV2Wire, NativePrefixFrameMetadataCodec>(
            spec, &codec,
        )
        .expect("prepare selected-wire TX");

    assert_eq!(prepared.payload_len(), VEHICLE_SIGNAL_V1_GOLDEN_BYTES.len());
    assert_eq!(prepared.payload_alignment(), 1);
    let decoded_metadata = codec
        .decode_frame_metadata(XcdrV2Wire::metadata_context(), prepared.encoded_metadata())
        .expect("decode prepared metadata");
    assert_eq!(decoded_metadata, metadata);

    let mut tx = UVecTxBuffer::with_alignment(
        prepared.metadata().clone(),
        prepared.payload_len(),
        prepared.payload_alignment(),
    )
    .expect("loan vector TX storage");
    XcdrV2Wire::encode_payload(&VEHICLE_SIGNAL_V1_GOLDEN_VALUE, tx.payload_mut())
        .expect("write serialized XCDRv2 bytes into loan");

    assert_eq!(tx.payload(), VEHICLE_SIGNAL_V1_GOLDEN_BYTES);
}

#[test]
fn public_trait_bounds_accept_supported_fixture() {
    fn assert_supported<W>()
    where
        W: up_rust::UWireEncode<VehicleSignalV1>
            + for<'a> up_rust::UWireDecode<'a, VehicleSignalV1>
            + up_rust::UWireReadDecode<VehicleSignalV1>,
    {
    }

    assert_supported::<XcdrV2Wire>();
}

#[test]
fn external_fixed_field_type_round_trips_through_final_traits() {
    let value = ExternalFixedPayload {
        sequence: 7,
        ready: true,
        values: [-1, 0, 44],
        checksum: 0x0123_4567_89ab_cdef,
    };
    let encoded = XcdrV2Payload::encode(&value).expect("encode external fixed-field payload");

    assert_eq!(encoded.as_bytes().len(), ExternalFixedPayload::ENCODED_LEN);
    assert_eq!(
        encoded.as_bytes(),
        [
            0x06, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xff, 0xff,
            0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0xef, 0xcd, 0xab, 0x89,
            0x67, 0x45, 0x23, 0x01,
        ]
    );
    assert_eq!(encoded.decode().expect("decode owned payload"), value);

    let decoded: ExternalFixedPayload = XcdrV2Wire::decode_payload_from_reader(
        std::io::Cursor::new(encoded.as_bytes()),
        encoded.as_bytes().len(),
        up_rust::PayloadDecodeLimit::new(ExternalFixedPayload::ENCODED_LEN),
    )
    .expect("reader decode external fixed-field payload");
    assert_eq!(decoded, value);

    let mut invalid_bool = encoded.into_bytes();
    invalid_bool[8] = 2;
    let result = ExternalFixedPayload::decode_xcdr_v2(&invalid_bool);
    assert!(matches!(
        result,
        Err(up_rust::UWireError::InvalidPayload(message))
            if message.contains("bool discriminant")
    ));
}

fn metadata() -> UFrameMetadata {
    let topic = UUri::try_from_parts("vehicle", 0x4210, 0x01, 0x9000).expect("topic URI");
    UFrameMetadata::publish(topic)
        .with_payload_encoding(XcdrV2Wire::payload_encoding())
        .build()
        .expect("metadata")
}
