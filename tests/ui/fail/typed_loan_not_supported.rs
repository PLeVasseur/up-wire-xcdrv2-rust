/********************************************************************************
 * Copyright (c) 2026 Contributors to the Eclipse Foundation
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

use up_rust::{LoanPayload, UWirePayload};
use up_wire_xcdrv2::{VehicleSignalV1, XcdrV2Wire};

fn needs_typed_loan<W>()
where
    W: UWirePayload<VehicleSignalV1>,
    <W as UWirePayload<VehicleSignalV1>>::Codec: LoanPayload<VehicleSignalV1>,
{
}

fn main() {
    needs_typed_loan::<XcdrV2Wire>();
}
