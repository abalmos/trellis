import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { exerciseDeviceState } from "./device_state.ts";

const CASE_ID = "state.typescript-device-rust-owner" as const;

liveTrellisTest({
  name:
    "state.typescript-device-rust-owner uses activated device State facades",
  scope: runtimeScopeForCase(CASE_ID),
  fn: exerciseDeviceState,
});
