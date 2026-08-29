import { defineAppContract } from "@qlever-llc/trellis/contracts";
import { AuthDeviceUserAuthoritiesResolve } from "@qlever-llc/trellis/sdk/auth";

export const contract = defineAppContract(() => ({
  id: "trellis.portal.activation@v1",
  apiId: "trellis.portal.activation@v1",
  displayName: "Trellis Device Activation",
  description:
    "Trellis built-in app for authenticated device activation over the Auth.DeviceUserAuthorities.Resolve operation.",
  uses: [AuthDeviceUserAuthoritiesResolve],
}));

export default contract;
