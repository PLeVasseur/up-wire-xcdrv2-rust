//! External XCDRv2 selected wire proof for the userializer first wave.
//!
//! This crate deliberately supports one crate-owned integer fixture. It proves
//! that production XCDRv2 can live outside `up-rust` while consuming only public
//! selected-wire traits and native-prefix metadata APIs.

use std::{io::Read, marker::PhantomData};

use up_rust::selected_wire_user_api::{UNativePrefixWireTransport, UWithNativePrefixWire};
use up_rust::wire_implementer_api::{
    UProtocolNativeWire, UWire, UWirePayload, WireIdentity, NATIVE_PREFIX_METADATA_LAYOUT_ID,
    XCDR_V2_PAYLOAD_FAMILY_ID, XCDR_V2_WIRE_ID,
};
use up_rust::{
    DecodePayload, EncodePayload, PayloadEncoding, PayloadFormat, PayloadLayout, ReadDecodePayload,
    UWireError,
};

const XCDR2_LE_FIXTURE_PREFIX: [u8; 4] = [0x06, 0x00, 0x00, 0x00];

/// Payload encoding id used by the constrained first-wave XCDRv2 fixture.
pub const XCDR_V2_ENCODING_ID: &str = "up.xcdr-v2";

/// Payload content type for the frozen `VehicleSignalV1` fixture.
pub const VEHICLE_SIGNAL_V1_CONTENT_TYPE: &str =
    "application/vnd.uprotocol.xcdr-v2;type=\"VehicleSignalV1\";endianness=little;version=2";

/// Frozen golden value used by USR-05X tests and downstream smoke proofs.
pub const VEHICLE_SIGNAL_V1_GOLDEN_VALUE: VehicleSignalV1 = VehicleSignalV1 {
    vehicle_id: 0x0000_1234,
    signal_id: 0x0007,
    sequence: 0x002a,
    value: -12345,
};

/// Frozen little-endian XCDRv2 fixture bytes for [`VEHICLE_SIGNAL_V1_GOLDEN_VALUE`].
pub const VEHICLE_SIGNAL_V1_GOLDEN_BYTES: [u8; VehicleSignalV1::ENCODED_LEN] = [
    0x06, 0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00, 0x07, 0x00, 0x2a, 0x00, 0xc7, 0xcf, 0xff, 0xff,
];

/// External selected wire marker for constrained XCDRv2 payloads.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct XcdrV2Wire;

/// Compatibility transport shape for XCDRv2 payloads with native-prefix metadata.
///
/// This composes XCDRv2 payload encoding with the shared native-prefix metadata
/// layout. A future real XCDRv2 metadata codec should use a distinct type and
/// metadata layout identity.
pub type XcdrV2NativePrefixTransport<TCore> = UNativePrefixWireTransport<TCore, XcdrV2Wire>;

/// Builds a compatibility transport for XCDRv2 payloads with native-prefix metadata.
#[must_use]
pub fn with_xcdr_v2_native_prefix<TCore>(core: TCore) -> XcdrV2NativePrefixTransport<TCore> {
    core.into_native_prefix_wire_transport(XcdrV2Wire)
}

impl UWire for XcdrV2Wire {
    const WIRE_ID: WireIdentity = XCDR_V2_WIRE_ID;
    const PAYLOAD_FAMILY_ID: WireIdentity = XCDR_V2_PAYLOAD_FAMILY_ID;
    const METADATA_LAYOUT_ID: WireIdentity = NATIVE_PREFIX_METADATA_LAYOUT_ID;
    const FORMAT_VERSION: u16 = UProtocolNativeWire::FORMAT_VERSION;
}

impl PayloadFormat for XcdrV2Wire {
    fn name() -> &'static str {
        "xcdr-v2-vehicle-signal-v1"
    }

    fn encoding() -> PayloadEncoding {
        PayloadEncoding::custom(XCDR_V2_ENCODING_ID, VEHICLE_SIGNAL_V1_CONTENT_TYPE)
            .expect("valid XCDRv2 payload encoding")
    }
}

impl UWirePayload<VehicleSignalV1> for XcdrV2Wire {
    type Codec = Self;
}

/// Types with explicit support in this constrained XCDRv2 fixture adapter.
pub trait XcdrV2Mappable: Sized {
    /// Stable fixture type name used in diagnostics and content type decisions.
    const TYPE_NAME: &'static str;
    /// Exact serialized length for this first-wave fixture encoding.
    const ENCODED_LEN: usize;

    /// Encodes `self` into the frozen little-endian XCDRv2 fixture bytes.
    fn encode_xcdr_v2(&self, dst: &mut [u8]) -> Result<(), UWireError>;

    /// Decodes `Self` from the frozen little-endian XCDRv2 fixture bytes.
    fn decode_xcdr_v2(src: &[u8]) -> Result<Self, UWireError>;
}

/// Owned encoded XCDRv2 payload bytes for a supported fixture type.
#[derive(Clone, Debug)]
pub struct XcdrV2Payload<T> {
    bytes: Vec<u8>,
    _payload: PhantomData<fn() -> T>,
}

impl<T> XcdrV2Payload<T>
where
    T: XcdrV2Mappable,
{
    /// Encodes a supported fixture value into owned XCDRv2 bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the fixture encoder rejects the value.
    pub fn encode(value: &T) -> Result<Self, UWireError> {
        let mut bytes = vec![0_u8; T::ENCODED_LEN];
        value.encode_xcdr_v2(&mut bytes)?;
        Ok(Self {
            bytes,
            _payload: PhantomData,
        })
    }

    /// Returns the encoded XCDRv2 bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Decodes the owned XCDRv2 bytes into the supported fixture type.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes are not the frozen fixture encoding.
    pub fn decode(&self) -> Result<T, UWireError> {
        T::decode_xcdr_v2(&self.bytes)
    }

    /// Consumes the wrapper and returns the encoded bytes.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// Crate-owned integer-only fixture frozen by the `USR-05X` preflight.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VehicleSignalV1 {
    /// Vehicle identifier.
    pub vehicle_id: u32,
    /// Signal identifier within the vehicle domain.
    pub signal_id: u16,
    /// Monotonic fixture sequence.
    pub sequence: u16,
    /// Signed signal value.
    pub value: i32,
}

impl VehicleSignalV1 {
    /// Serialized size of the constrained first-wave XCDRv2 fixture.
    pub const ENCODED_LEN: usize = 16;
}

impl XcdrV2Mappable for VehicleSignalV1 {
    const TYPE_NAME: &'static str = "VehicleSignalV1";
    const ENCODED_LEN: usize = Self::ENCODED_LEN;

    fn encode_xcdr_v2(&self, dst: &mut [u8]) -> Result<(), UWireError> {
        if dst.len() < Self::ENCODED_LEN {
            return Err(UWireError::buffer_too_small(Self::ENCODED_LEN, dst.len()));
        }

        dst[..Self::ENCODED_LEN].fill(0);
        dst[0..4].copy_from_slice(&XCDR2_LE_FIXTURE_PREFIX);
        dst[4..8].copy_from_slice(&self.vehicle_id.to_le_bytes());
        dst[8..10].copy_from_slice(&self.signal_id.to_le_bytes());
        dst[10..12].copy_from_slice(&self.sequence.to_le_bytes());
        dst[12..16].copy_from_slice(&self.value.to_le_bytes());
        Ok(())
    }

    fn decode_xcdr_v2(src: &[u8]) -> Result<Self, UWireError> {
        if src.len() != Self::ENCODED_LEN {
            return Err(UWireError::invalid_payload_length(
                Self::ENCODED_LEN,
                src.len(),
            ));
        }
        if src[0..4] != XCDR2_LE_FIXTURE_PREFIX {
            return Err(UWireError::invalid_payload(
                "unsupported XCDRv2 fixture prefix or endian/version",
            ));
        }

        Ok(Self {
            vehicle_id: read_u32(src, 4),
            signal_id: read_u16(src, 8),
            sequence: read_u16(src, 10),
            value: read_i32(src, 12),
        })
    }
}

impl<T> EncodePayload<T> for XcdrV2Wire
where
    T: XcdrV2Mappable,
{
    fn payload_layout(_value: &T) -> Result<PayloadLayout, UWireError> {
        PayloadLayout::new(T::ENCODED_LEN, 1)
    }

    fn encode_payload(value: &T, dst: &mut [u8]) -> Result<(), UWireError> {
        value.encode_xcdr_v2(dst)
    }
}

impl<'a, T> DecodePayload<'a, T> for XcdrV2Wire
where
    T: XcdrV2Mappable,
{
    fn decode_payload(src: &'a [u8]) -> Result<T, UWireError> {
        T::decode_xcdr_v2(src)
    }
}

impl<T> ReadDecodePayload<T> for XcdrV2Wire
where
    T: XcdrV2Mappable,
{
    fn decode_payload_from_reader<R: Read>(
        mut reader: R,
        payload_len: usize,
    ) -> Result<T, UWireError> {
        if payload_len != T::ENCODED_LEN {
            return Err(UWireError::invalid_payload_length(
                T::ENCODED_LEN,
                payload_len,
            ));
        }
        let mut bytes = vec![0_u8; payload_len];
        reader
            .read_exact(&mut bytes)
            .map_err(|error| UWireError::invalid_payload(error.to_string()))?;
        T::decode_xcdr_v2(&bytes)
    }
}

fn read_u16(src: &[u8], offset: usize) -> u16 {
    let mut bytes = [0_u8; 2];
    bytes.copy_from_slice(&src[offset..offset + 2]);
    u16::from_le_bytes(bytes)
}

fn read_u32(src: &[u8], offset: usize) -> u32 {
    let mut bytes = [0_u8; 4];
    bytes.copy_from_slice(&src[offset..offset + 4]);
    u32::from_le_bytes(bytes)
}

fn read_i32(src: &[u8], offset: usize) -> i32 {
    let mut bytes = [0_u8; 4];
    bytes.copy_from_slice(&src[offset..offset + 4]);
    i32::from_le_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use up_rust::wire_implementer_api::{NativePrefixProtobufMetadataCodec, UWireMetadataCodec};
    use up_rust::PayloadCodec;

    #[test]
    fn vehicle_signal_golden_bytes_are_frozen() {
        let encoded = XcdrV2Wire::encode_payload_owned(&VEHICLE_SIGNAL_V1_GOLDEN_VALUE)
            .expect("encode golden fixture");

        assert_eq!(encoded.as_ref(), VEHICLE_SIGNAL_V1_GOLDEN_BYTES);
    }

    #[test]
    fn owned_and_reader_decode_round_trip_supported_fixture() {
        let encoded = XcdrV2Payload::<VehicleSignalV1>::encode(&VEHICLE_SIGNAL_V1_GOLDEN_VALUE)
            .expect("encode owned payload");

        assert_eq!(encoded.as_bytes(), VEHICLE_SIGNAL_V1_GOLDEN_BYTES);
        assert_eq!(
            encoded.decode().expect("decode owned payload"),
            VEHICLE_SIGNAL_V1_GOLDEN_VALUE
        );

        let decoded: VehicleSignalV1 = XcdrV2Wire::decode_payload_from_reader(
            Cursor::new(encoded.as_bytes()),
            encoded.as_bytes().len(),
        )
        .expect("decode from reader");
        assert_eq!(decoded, VEHICLE_SIGNAL_V1_GOLDEN_VALUE);
    }

    #[test]
    fn rejects_wrong_length_and_prefix() {
        let too_short = &VEHICLE_SIGNAL_V1_GOLDEN_BYTES[..15];
        let too_short_result: Result<VehicleSignalV1, UWireError> =
            XcdrV2Wire::decode_payload(too_short);
        assert!(matches!(
            too_short_result,
            Err(UWireError::InvalidPayload(_))
        ));

        let mut wrong_prefix = VEHICLE_SIGNAL_V1_GOLDEN_BYTES;
        wrong_prefix[0] = 0xff;
        let wrong_prefix_result: Result<VehicleSignalV1, UWireError> =
            XcdrV2Wire::decode_payload(&wrong_prefix);
        assert!(matches!(
            wrong_prefix_result,
            Err(UWireError::InvalidPayload(message)) if message.contains("prefix")
        ));
    }

    #[test]
    fn metadata_uses_public_up_rust_native_prefix_api() {
        assert_eq!(XcdrV2Wire::WIRE_ID, XCDR_V2_WIRE_ID);
        assert_eq!(XcdrV2Wire::PAYLOAD_FAMILY_ID, XCDR_V2_PAYLOAD_FAMILY_ID);

        let metadata = crate_metadata();
        let encoded = NativePrefixProtobufMetadataCodec
            .encode_frame_metadata(XcdrV2Wire::metadata_context(), &metadata)
            .expect("encode metadata");
        let decoded = NativePrefixProtobufMetadataCodec
            .decode_frame_metadata(XcdrV2Wire::metadata_context(), &encoded)
            .expect("decode metadata");

        assert_eq!(decoded, metadata);
        assert_eq!(
            decoded.payload_encoding(),
            Some(&XcdrV2Wire::payload_encoding())
        );
    }

    fn crate_metadata() -> up_rust::UFrameMetadata {
        let topic =
            up_rust::UUri::try_from_parts("vehicle", 0x4210, 0x01, 0x9000).expect("topic URI");
        let message = up_rust::UMessageBuilder::publish(topic)
            .build()
            .expect("message");
        up_rust::UFrameMetadata::new(
            message.attributes().clone(),
            Some(XcdrV2Wire::payload_encoding()),
        )
        .expect("metadata")
    }
}
