/********************************************************************************
 * Copyright (c) 2026 Contributors to the Eclipse Foundation
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

use up_rust::UWireEncode;
use up_wire_xcdrv2::XcdrV2Wire;

struct UnsupportedStringPayload {
    value: String,
}

fn needs_xcdrv2_encode<W>()
where
    W: UWireEncode<UnsupportedStringPayload>,
{
}

fn main() {
    needs_xcdrv2_encode::<XcdrV2Wire>();
}
