# up-wire-xcdrv2-rust

External fixed-field `XcdrV2Wire` crate for the userializer first wave.

`#[derive(up_wire_xcdrv2::XcdrV2Type)]` maps an external named-field struct
whose fields are fixed-size scalars or fixed arrays into little-endian
PLAIN_CDR2 bytes. The crate also retains the constrained `VehicleSignalV1`
fixture and uses only public `up-rust` selected-wire traits and native-prefix
metadata APIs.

## Deployment identity

Selecting `XcdrV2Wire` opts into this profile's deployment-private payload encoding
`0xF001` for the constrained PLAIN_CDR2 contract above. Peers must agree to that
assignment and reserve it consistently with all other private encodings, including
native-profile tables. It is not a public uProtocol registry allocation; the
earlier proposed public ID 9 is unassigned and is no longer emitted.

The candidate deployment reserves `0xF001`/`0xF002`/`0xF003` for XCDRv2/Arrow/OMG
IDL serialized profiles respectively. Native structural type tokens are separate
metadata and are not produced by this serialized payload codec.

Dynamic strings, vectors, optionals, unions, and broad XTypes/codegen support
remain intentionally unsupported.
