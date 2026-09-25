/********************************************************************************
 * Copyright (c) 2026 Contributors to the Eclipse Foundation
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

use up_wire_xcdrv2::XcdrV2Type;

#[derive(XcdrV2Type)]
struct UnsupportedDynamicFields {
    name: String,
    values: Vec<i32>,
    checksum: Option<u32>,
}

fn main() {}
