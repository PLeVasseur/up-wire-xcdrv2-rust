/********************************************************************************
 * Copyright (c) 2026 Contributors to the Eclipse Foundation
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

#[test]
fn xcdrv2_wire_trybuild() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/pass/supported_fixture.rs");
    tests.pass("tests/ui/pass/derived_fixed_fields.rs");
    tests.compile_fail("tests/ui/fail/unsupported_string_payload.rs");
    tests.compile_fail("tests/ui/fail/unsupported_dynamic_fields.rs");
    tests.compile_fail("tests/ui/fail/unsupported_enum_derive.rs");
    tests.compile_fail("tests/ui/fail/unsupported_tuple_struct_derive.rs");
}
