# up-wire-xcdrv2-rust

External fixed-field `XcdrV2Wire` crate for the userializer first wave.

`#[derive(up_wire_xcdrv2::XcdrV2Type)]` maps an external named-field struct
whose fields are fixed-size scalars or fixed arrays into little-endian
PLAIN_CDR2 bytes. The crate also retains the constrained `VehicleSignalV1`
fixture and uses only public `up-rust` selected-wire traits and native-prefix
metadata APIs.

Dynamic strings, vectors, optionals, unions, and broad XTypes/codegen support
remain intentionally unsupported.
