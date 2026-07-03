/********************************************************************************
 * Copyright (c) 2026 Contributors to the Eclipse Foundation
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

use up_rust::transport_implementer_api::PreparedTxLoanSpec;
use up_rust::wire_implementer_api::{NativePrefixProtobufMetadataCodec, UWire, UWireMetadataCodec};
use up_rust::{
    EncodePayload, PayloadCodec, UFrameMetadata, UMessageBuilder, UTxBuffer, UTxLoanSpec, UUri,
    UVecTxBuffer, ValidatedTxLoanSpec,
};
use up_wire_xcdrv2::{
    VehicleSignalV1, XcdrV2Wire, VEHICLE_SIGNAL_V1_GOLDEN_BYTES, VEHICLE_SIGNAL_V1_GOLDEN_VALUE,
};

#[test]
fn serialized_zero_copy_tx_fixture_writes_xcdrv2_bytes_into_loan() {
    let metadata = metadata();
    let layout = XcdrV2Wire::payload_layout(&VEHICLE_SIGNAL_V1_GOLDEN_VALUE)
        .expect("measure XCDRv2 payload");
    let spec = UTxLoanSpec::payload(metadata.clone(), layout.len(), layout.align())
        .expect("create TX loan spec");
    let prepared =
        PreparedTxLoanSpec::from_validated::<XcdrV2Wire, NativePrefixProtobufMetadataCodec>(
            ValidatedTxLoanSpec::try_from(spec).expect("validate TX loan spec"),
            &NativePrefixProtobufMetadataCodec,
        )
        .expect("prepare selected-wire TX");

    assert_eq!(prepared.payload_len(), VEHICLE_SIGNAL_V1_GOLDEN_BYTES.len());
    assert_eq!(prepared.payload_alignment_proof().as_usize(), 1);
    let decoded_metadata = NativePrefixProtobufMetadataCodec
        .decode_frame_metadata(XcdrV2Wire::metadata_context(), prepared.encoded_metadata())
        .expect("decode prepared metadata");
    assert_eq!(decoded_metadata, metadata);

    let mut tx = UVecTxBuffer::with_alignment(
        prepared.metadata().clone(),
        prepared.payload_len(),
        prepared.payload_alignment_proof().as_usize(),
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
        W: up_rust::wire_implementer_api::UWire
            + up_rust::wire_implementer_api::UWireEncode<VehicleSignalV1>
            + for<'a> up_rust::wire_implementer_api::UWireDecode<'a, VehicleSignalV1>
            + up_rust::wire_implementer_api::UWireReadDecode<VehicleSignalV1>,
    {
    }

    assert_supported::<XcdrV2Wire>();
}

fn metadata() -> UFrameMetadata {
    let topic = UUri::try_from_parts("vehicle", 0x4210, 0x01, 0x9000).expect("topic URI");
    let message = UMessageBuilder::publish(topic).build().expect("message");
    UFrameMetadata::new(
        message.attributes().clone(),
        Some(XcdrV2Wire::payload_encoding()),
    )
    .expect("metadata")
}
