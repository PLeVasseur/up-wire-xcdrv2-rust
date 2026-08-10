/********************************************************************************
 * Copyright (c) 2026 Contributors to the Eclipse Foundation
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

use up_rust::{
    EncodePayload, NativePrefixFrameMetadataCodec, PayloadCodec, PreparedTxLoanSpec,
    UFrameMetadata, UTxBuffer, UTxLoanSpec, UUri, UVecTxBuffer, UWire, UWireMetadataCodec,
};
use up_wire_xcdrv2::{
    VehicleSignalV1, XcdrV2Wire, VEHICLE_SIGNAL_V1_GOLDEN_BYTES, VEHICLE_SIGNAL_V1_GOLDEN_VALUE,
};

static_assertions::assert_not_impl_any!(XcdrV2Wire: up_rust::BorrowPayload<VehicleSignalV1>);

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

fn metadata() -> UFrameMetadata {
    let topic = UUri::try_from_parts("vehicle", 0x4210, 0x01, 0x9000).expect("topic URI");
    UFrameMetadata::publish(topic)
        .with_payload_encoding(XcdrV2Wire::payload_encoding())
        .build()
        .expect("metadata")
}
