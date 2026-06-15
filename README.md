# up-wire-xcdrv2-rust

External `XcdrV2Wire` proof crate for the userializer first wave.

This crate intentionally implements only the constrained `VehicleSignalV1`
little-endian XCDRv2 fixture adapter needed by `USR-05X`. It uses public
`up-rust` selected-wire traits and native-prefix metadata APIs only; broad
XTypes/codegen support is out of first-wave scope.
