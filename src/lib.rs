//! Fixed-field XCDRv2 selected-wire support for typed uProtocol payloads.
//!
//! External named-field structs can derive [`XcdrV2Type`] when every field is a
//! supported fixed-size scalar or fixed array.
//! Dynamic fields such as `String`, `Vec`, and `Option` are intentionally not
//! supported.

extern crate self as up_wire_xcdrv2;

use std::{convert::TryInto, io::Read, marker::PhantomData};

use up_rust::{
    DecodePayload, EncodePayload, PayloadCodecIdentity, PayloadDecodeLimit, PayloadEncoding,
    PayloadLayout, ReadDecodePayload, UWire, UWireError, UWirePayload, WireIdentity,
    NATIVE_PREFIX_METADATA_LAYOUT_ID, XCDR_V2_PAYLOAD_FAMILY_ID, XCDR_V2_WIRE_ID,
};

const XCDR2_LE_FIXTURE_PREFIX: [u8; 4] = [0x06, 0x00, 0x00, 0x00];

pub use up_wire_xcdrv2_macros::XcdrV2Type;

/// Encapsulation size for little-endian PLAIN_CDR2 payloads.
#[doc(hidden)]
pub const XCDR_V2_ENCAPSULATION_LEN: usize = XCDR2_LE_FIXTURE_PREFIX.len();

/// Payload encoding id used by the constrained first-wave XCDRv2 fixture.
pub const XCDR_V2_ENCODING_ID: u32 = 9;

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

impl UWire for XcdrV2Wire {
    const WIRE_ID: WireIdentity = XCDR_V2_WIRE_ID;
    const PAYLOAD_FAMILY_ID: WireIdentity = XCDR_V2_PAYLOAD_FAMILY_ID;
    const METADATA_LAYOUT_ID: WireIdentity = NATIVE_PREFIX_METADATA_LAYOUT_ID;
    const FORMAT_VERSION: u16 = up_rust::wire::FORMAT_VERSION;
}

impl PayloadCodecIdentity for XcdrV2Wire {
    fn name() -> &'static str {
        "xcdr-v2-vehicle-signal-v1"
    }

    fn encoding() -> PayloadEncoding {
        PayloadEncoding::from_registry_entry(XCDR_V2_ENCODING_ID)
    }
}

impl<T> UWirePayload<T> for XcdrV2Wire
where
    T: XcdrV2Mappable,
{
    type Codec = Self;
}

/// Fixed-size types with an explicit XCDRv2 mapping.
pub trait XcdrV2Mappable: Sized {
    /// Stable type name used in diagnostics and content type decisions.
    const TYPE_NAME: &'static str;
    /// Exact serialized length, including the PLAIN_CDR2 encapsulation.
    const ENCODED_LEN: usize;

    /// Encodes `self` into little-endian PLAIN_CDR2 bytes.
    fn encode_xcdr_v2(&self, dst: &mut [u8]) -> Result<(), UWireError>;

    /// Decodes `Self` from little-endian PLAIN_CDR2 bytes.
    fn decode_xcdr_v2(src: &[u8]) -> Result<Self, UWireError>;
}

/// Marker contract implemented by [`XcdrV2Type`] for supported external structs.
pub trait XcdrV2Type: XcdrV2Mappable {}

impl<T> XcdrV2Type for T where T: XcdrV2Mappable {}

/// Fixed-size field contract used by [`XcdrV2Type`].
#[doc(hidden)]
pub trait XcdrV2Field: sealed::Sealed + Sized {
    /// CDR field alignment.
    const ALIGNMENT: usize;
    /// Exact encoded field size.
    const ENCODED_LEN: usize;

    /// Encodes one field.
    fn encode_field(&self, encoder: &mut XcdrV2Encoder<'_>) -> Result<(), UWireError>;

    /// Decodes one field.
    fn decode_field(decoder: &mut XcdrV2Decoder<'_>) -> Result<Self, UWireError>;
}

/// Internal fixed-size little-endian XCDRv2 encoder used by the derive.
#[doc(hidden)]
#[derive(Debug)]
pub struct XcdrV2Encoder<'a> {
    dst: &'a mut [u8],
    offset: usize,
}

impl<'a> XcdrV2Encoder<'a> {
    /// Creates an encoder for one exact-size payload.
    pub fn new(dst: &'a mut [u8], encoded_len: usize) -> Result<Self, UWireError> {
        if dst.len() < encoded_len {
            return Err(UWireError::buffer_too_small(encoded_len, dst.len()));
        }
        dst[..encoded_len].fill(0);
        Ok(Self {
            dst: &mut dst[..encoded_len],
            offset: 0,
        })
    }

    /// Writes the PLAIN_CDR2 little-endian encapsulation.
    pub fn write_encapsulation(&mut self) -> Result<(), UWireError> {
        self.write_bytes(&XCDR2_LE_FIXTURE_PREFIX)
    }

    /// Writes fixed-width bytes after applying CDR alignment.
    pub fn write_aligned_bytes(
        &mut self,
        alignment: usize,
        bytes: &[u8],
    ) -> Result<(), UWireError> {
        self.align(alignment)?;
        self.write_bytes(bytes)
    }

    /// Writes raw bytes at the current position.
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), UWireError> {
        let end = self
            .offset
            .checked_add(bytes.len())
            .ok_or_else(|| UWireError::serialization_error("XCDRv2 offset overflow"))?;
        if end > self.dst.len() {
            return Err(UWireError::buffer_too_small(end, self.dst.len()));
        }
        self.dst[self.offset..end].copy_from_slice(bytes);
        self.offset = end;
        Ok(())
    }

    /// Verifies that the exact fixed-size payload was produced.
    pub fn finish(self) -> Result<(), UWireError> {
        if self.offset != self.dst.len() {
            return Err(UWireError::serialization_error(format!(
                "XCDRv2 encoded {} bytes, expected {}",
                self.offset,
                self.dst.len()
            )));
        }
        Ok(())
    }

    fn align(&mut self, alignment: usize) -> Result<(), UWireError> {
        if !alignment.is_power_of_two() {
            return Err(UWireError::serialization_error(format!(
                "invalid XCDRv2 alignment {alignment}"
            )));
        }
        let padding = padding_for(self.offset, alignment);
        let end = self
            .offset
            .checked_add(padding)
            .ok_or_else(|| UWireError::serialization_error("XCDRv2 offset overflow"))?;
        if end > self.dst.len() {
            return Err(UWireError::buffer_too_small(end, self.dst.len()));
        }
        self.offset = end;
        Ok(())
    }
}

/// Internal fixed-size little-endian XCDRv2 decoder used by the derive.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct XcdrV2Decoder<'a> {
    src: &'a [u8],
    offset: usize,
}

impl<'a> XcdrV2Decoder<'a> {
    /// Creates a decoder for one exact-size payload.
    pub fn new(src: &'a [u8], encoded_len: usize) -> Result<Self, UWireError> {
        if src.len() != encoded_len {
            return Err(UWireError::invalid_payload_length(encoded_len, src.len()));
        }
        Ok(Self { src, offset: 0 })
    }

    /// Reads and validates the PLAIN_CDR2 little-endian encapsulation.
    pub fn read_encapsulation(&mut self) -> Result<(), UWireError> {
        if self.read_exact(XCDR_V2_ENCAPSULATION_LEN)? != XCDR2_LE_FIXTURE_PREFIX {
            return Err(UWireError::invalid_payload(
                "unsupported XCDRv2 fixture prefix or endian/version",
            ));
        }
        Ok(())
    }

    /// Reads fixed-width bytes after applying CDR alignment.
    pub fn read_aligned_bytes(
        &mut self,
        alignment: usize,
        len: usize,
    ) -> Result<&'a [u8], UWireError> {
        self.align(alignment)?;
        self.read_exact(len)
    }

    /// Verifies that the entire fixed-size payload was consumed.
    pub fn finish(self) -> Result<(), UWireError> {
        if self.offset != self.src.len() {
            return Err(UWireError::invalid_payload(format!(
                "XCDRv2 payload has {} trailing bytes",
                self.src.len() - self.offset
            )));
        }
        Ok(())
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], UWireError> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or_else(|| UWireError::invalid_payload("XCDRv2 offset overflow"))?;
        if end > self.src.len() {
            return Err(UWireError::invalid_payload(format!(
                "XCDRv2 payload ended at byte {}, needed byte {end}",
                self.src.len()
            )));
        }
        let bytes = &self.src[self.offset..end];
        self.offset = end;
        Ok(bytes)
    }

    fn align(&mut self, alignment: usize) -> Result<(), UWireError> {
        if !alignment.is_power_of_two() {
            return Err(UWireError::serialization_error(format!(
                "invalid XCDRv2 alignment {alignment}"
            )));
        }
        let padding = padding_for(self.offset, alignment);
        self.offset = self
            .offset
            .checked_add(padding)
            .ok_or_else(|| UWireError::invalid_payload("XCDRv2 offset overflow"))?;
        if self.offset > self.src.len() {
            return Err(UWireError::invalid_payload(
                "XCDRv2 padding exceeds payload length",
            ));
        }
        Ok(())
    }
}

macro_rules! xcdr_scalar {
    ($ty:ty, $alignment:expr, $to_bytes:ident, $from_bytes:ident) => {
        impl sealed::Sealed for $ty {}

        impl XcdrV2Field for $ty {
            const ALIGNMENT: usize = $alignment;
            const ENCODED_LEN: usize = ::core::mem::size_of::<$ty>();

            fn encode_field(&self, encoder: &mut XcdrV2Encoder<'_>) -> Result<(), UWireError> {
                encoder.write_aligned_bytes($alignment, &self.$to_bytes())
            }

            fn decode_field(decoder: &mut XcdrV2Decoder<'_>) -> Result<Self, UWireError> {
                let bytes = decoder.read_aligned_bytes($alignment, Self::ENCODED_LEN)?;
                Ok(<$ty>::$from_bytes(
                    bytes.try_into().expect("fixed-width XCDRv2 scalar"),
                ))
            }
        }
    };
}

xcdr_scalar!(u16, 2, to_le_bytes, from_le_bytes);
xcdr_scalar!(i16, 2, to_le_bytes, from_le_bytes);
xcdr_scalar!(u32, 4, to_le_bytes, from_le_bytes);
xcdr_scalar!(i32, 4, to_le_bytes, from_le_bytes);
xcdr_scalar!(u64, 8, to_le_bytes, from_le_bytes);
xcdr_scalar!(i64, 8, to_le_bytes, from_le_bytes);
xcdr_scalar!(f32, 4, to_le_bytes, from_le_bytes);
xcdr_scalar!(f64, 8, to_le_bytes, from_le_bytes);

impl sealed::Sealed for u8 {}

impl XcdrV2Field for u8 {
    const ALIGNMENT: usize = 1;
    const ENCODED_LEN: usize = 1;

    fn encode_field(&self, encoder: &mut XcdrV2Encoder<'_>) -> Result<(), UWireError> {
        encoder.write_bytes(&[*self])
    }

    fn decode_field(decoder: &mut XcdrV2Decoder<'_>) -> Result<Self, UWireError> {
        Ok(decoder.read_exact(1)?[0])
    }
}

impl sealed::Sealed for i8 {}

impl XcdrV2Field for i8 {
    const ALIGNMENT: usize = 1;
    const ENCODED_LEN: usize = 1;

    fn encode_field(&self, encoder: &mut XcdrV2Encoder<'_>) -> Result<(), UWireError> {
        (*self as u8).encode_field(encoder)
    }

    fn decode_field(decoder: &mut XcdrV2Decoder<'_>) -> Result<Self, UWireError> {
        Ok(u8::decode_field(decoder)? as i8)
    }
}

impl sealed::Sealed for bool {}

impl XcdrV2Field for bool {
    const ALIGNMENT: usize = 1;
    const ENCODED_LEN: usize = 1;

    fn encode_field(&self, encoder: &mut XcdrV2Encoder<'_>) -> Result<(), UWireError> {
        u8::from(*self).encode_field(encoder)
    }

    fn decode_field(decoder: &mut XcdrV2Decoder<'_>) -> Result<Self, UWireError> {
        match u8::decode_field(decoder)? {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(UWireError::invalid_payload(format!(
                "invalid XCDRv2 bool discriminant {other}"
            ))),
        }
    }
}

impl<T, const N: usize> sealed::Sealed for [T; N] where T: XcdrV2Field {}

impl<T, const N: usize> XcdrV2Field for [T; N]
where
    T: XcdrV2Field,
{
    const ALIGNMENT: usize = if N == 0 { 1 } else { T::ALIGNMENT };
    const ENCODED_LEN: usize = T::ENCODED_LEN * N;

    fn encode_field(&self, encoder: &mut XcdrV2Encoder<'_>) -> Result<(), UWireError> {
        for value in self {
            value.encode_field(encoder)?;
        }
        Ok(())
    }

    fn decode_field(decoder: &mut XcdrV2Decoder<'_>) -> Result<Self, UWireError> {
        let mut values = Vec::with_capacity(N);
        for _ in 0..N {
            values.push(T::decode_field(decoder)?);
        }
        values.try_into().map_err(|_| {
            UWireError::serialization_error("internal XCDRv2 array decode length mismatch")
        })
    }
}

/// Calculates the end offset of one fixed-size aligned field.
#[doc(hidden)]
#[must_use]
pub const fn xcdr_v2_field_end(position: usize, alignment: usize, len: usize) -> usize {
    position + padding_for(position, alignment) + len
}

const fn padding_for(position: usize, alignment: usize) -> usize {
    let remainder = position % alignment;
    if remainder == 0 {
        0
    } else {
        alignment - remainder
    }
}

mod sealed {
    pub trait Sealed {}
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

/// Encoded-input policy for the fixed-size `VehicleSignalV1` reader path.
pub const VEHICLE_SIGNAL_V1_DECODE_LIMIT: PayloadDecodeLimit =
    PayloadDecodeLimit::new(VehicleSignalV1::ENCODED_LEN);

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
        limit: PayloadDecodeLimit,
    ) -> Result<T, UWireError> {
        if payload_len > limit.max_payload_bytes() {
            return Err(UWireError::invalid_payload(format!(
                "advertised payload length {payload_len} exceeds configured input limit {}",
                limit.max_payload_bytes()
            )));
        }
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
        let mut overrun = [0_u8; 1];
        match reader.read(&mut overrun) {
            Ok(0) => {}
            Ok(_) => {
                return Err(UWireError::invalid_payload(
                    "payload reader yielded bytes beyond the advertised length",
                ));
            }
            Err(error) => return Err(UWireError::invalid_payload(error.to_string())),
        }
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
    use up_rust::{NativePrefixFrameMetadataCodec, PayloadCodec, UWireMetadataCodec};

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
            VEHICLE_SIGNAL_V1_DECODE_LIMIT,
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
    fn reader_policy_rejects_limit_eof_and_overrun() {
        let below_contract: Result<VehicleSignalV1, UWireError> =
            XcdrV2Wire::decode_payload_from_reader(
                Cursor::new(VEHICLE_SIGNAL_V1_GOLDEN_BYTES),
                VehicleSignalV1::ENCODED_LEN,
                PayloadDecodeLimit::new(VehicleSignalV1::ENCODED_LEN - 1),
            );
        assert!(matches!(
            below_contract,
            Err(UWireError::InvalidPayload(message)) if message.contains("configured input limit")
        ));

        let early_eof: Result<VehicleSignalV1, UWireError> = XcdrV2Wire::decode_payload_from_reader(
            Cursor::new(&VEHICLE_SIGNAL_V1_GOLDEN_BYTES[..15]),
            VehicleSignalV1::ENCODED_LEN,
            VEHICLE_SIGNAL_V1_DECODE_LIMIT,
        );
        assert!(matches!(early_eof, Err(UWireError::InvalidPayload(_))));

        let mut overlong = VEHICLE_SIGNAL_V1_GOLDEN_BYTES.to_vec();
        overlong.push(0);
        let overrun: Result<VehicleSignalV1, UWireError> = XcdrV2Wire::decode_payload_from_reader(
            Cursor::new(overlong),
            VehicleSignalV1::ENCODED_LEN,
            VEHICLE_SIGNAL_V1_DECODE_LIMIT,
        );
        assert!(matches!(
            overrun,
            Err(UWireError::InvalidPayload(message)) if message.contains("beyond")
        ));
    }

    #[test]
    fn metadata_uses_public_up_rust_native_prefix_api() {
        assert_eq!(XcdrV2Wire::WIRE_ID, XCDR_V2_WIRE_ID);
        assert_eq!(XcdrV2Wire::PAYLOAD_FAMILY_ID, XCDR_V2_PAYLOAD_FAMILY_ID);

        let metadata = crate_metadata();
        let codec = NativePrefixFrameMetadataCodec;
        let encoded = codec
            .encode_frame_metadata(XcdrV2Wire::metadata_context(), &metadata)
            .expect("encode metadata");
        let decoded = codec
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
        up_rust::UFrameMetadata::publish(topic)
            .with_payload_encoding(XcdrV2Wire::payload_encoding())
            .build()
            .expect("metadata")
    }
}
