import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { exerciseDeviceState } from "./device_state.ts";

const CASE_ID = "state.activated-devices-rust-owner" as const;

liveTrellisTest({
  name: "state.activated-devices-rust-owner uses activated device clients",
  scope: runtimeScopeForCase(CASE_ID),
  fn: exerciseDeviceState,
});
