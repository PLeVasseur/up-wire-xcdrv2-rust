/********************************************************************************
 * Copyright (c) 2026 Contributors to the Eclipse Foundation
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

use up_rust::transport_implementer_api::PreparedTxLoanSpec;
use up_rust::wire_implementer_api::{NativePrefixFrameMetadataCodec, UWire, UWireMetadataCodec};
use up_rust::{
    EncodePayload, PayloadCodec, UFrameMetadata, UTxBuffer, UTxLoanSpec, UUri, UVecTxBuffer,
};
use up_wire_xcdrv2::{
    VehicleSignalV1, XcdrV2Wire, VEHICLE_SIGNAL_V1_GOLDEN_BYTES, VEHICLE_SIGNAL_V1_GOLDEN_VALUE,
};

static_assertions::assert_not_impl_any!(XcdrV2Wire: up_rust::LoanPayload<VehicleSignalV1>);

#[test]
fn serialized_zero_copy_tx_fixture_writes_xcdrv2_bytes_into_loan() {
    let metadata = metadata();
    let layout = XcdrV2Wire::payload_layout(&VEHICLE_SIGNAL_V1_GOLDEN_VALUE)
        .expect("measure XCDRv2 payload");
    let spec = UTxLoanSpec::payload(metadata.clone(), layout.len(), layout.align())
        .expect("create TX loan spec");
    let prepared =
        PreparedTxLoanSpec::from_validated::<XcdrV2Wire, NativePrefixFrameMetadataCodec>(
            spec,
            &NativePrefixFrameMetadataCodec,
        )
        .expect("prepare selected-wire TX");

    assert_eq!(prepared.payload_len(), VEHICLE_SIGNAL_V1_GOLDEN_BYTES.len());
    assert_eq!(prepared.payload_alignment_proof().as_usize(), 1);
    let decoded_metadata = NativePrefixFrameMetadataCodec
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
    UFrameMetadata::publish(topic)
        .with_payload_encoding(XcdrV2Wire::payload_encoding())
        .build()
        .expect("metadata")
}
