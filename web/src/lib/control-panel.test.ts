import { deepEqual, ok } from "node:assert/strict";
import { apis } from "trellis-web-generated";

import {
  canAccessRoute,
  getPageTitle,
  getVisibleNavSections,
} from "./control-panel.ts";

declare const Deno: {
  test(name: string, fn: () => void | Promise<void>): void;
};

Deno.test("control panel keeps admin navigation focused on active sections", () => {
  const sections = getVisibleNavSections({
    active: true,
    capabilities: [
      `${apis.auth.API.identity}::authorities_read`,
      `${apis.auth.API.identity}::capabilities_read`,
      `${apis.auth.API.identity}::devices_read`,
      `${apis.auth.API.identity}::portals_read`,
      `${apis.auth.API.identity}::services_read`,
      `${apis.auth.API.identity}::sessions_read`,
      `${apis.auth.API.identity}::users_read`,
      `${apis.events.API.identity}::read`,
      `${apis.health.API.identity}::read`,
      `${apis.jobs.API.identity}::read`,
    ],
    email: "ada@example.com",
    id: "user-1",
    name: "Ada",
    origin: "github",
  });
  const labels = sections.flatMap((section) =>
    section.items.map((item) => item.label)
  );
  const hrefs = sections.flatMap((section) =>
    section.items.map((item) => item.href)
  );
  const operateSection = sections.find((section) =>
    section.title === "Operate"
  );
  const manageSection = sections.find((section) => section.title === "Manage");

  deepEqual(operateSection?.items.map((item) => item.label), [
    "Overview",
    "Health Events",
    "Sessions",
    "Events",
    "Jobs",
    "Grants",
    "Capability Groups",
    "Portals",
  ]);

  deepEqual(manageSection?.items[0], {
    href: "/admin/services",
    label: "Services",
    icon: "server",
    capabilities: [`${apis.auth.API.identity}::services_read`],
  });
  deepEqual(manageSection?.items[1], {
    href: "/admin/devices",
    label: "Devices",
    icon: "phone",
    capabilities: [`${apis.auth.API.identity}::devices_read`],
  });
  deepEqual(manageSection?.items[2], {
    href: "/admin/users",
    label: "Users",
    icon: "users",
    capabilities: [`${apis.auth.API.identity}::users_read`],
  });
  ok(labels.includes("Jobs"));
  ok(!labels.includes("API Catalog"));
  ok(labels.includes("Account"));
  ok(!labels.includes("Settings"));
  ok(labels.includes("Grants"));
  ok(!labels.includes("Deployments"));
  ok(labels.includes("Devices"));
  ok(!labels.includes("Authority"));
  ok(hrefs.includes("/admin/services"));
  ok(hrefs.includes("/admin/devices"));
  ok(!hrefs.map(String).includes("/admin/authority"));
  ok(!hrefs.map(String).includes("/admin/deployments"));
  ok(!hrefs.map(String).includes("/admin/app-grants"));
  ok(!hrefs.map(String).includes("/admin/services/instances"));
  ok(!hrefs.map(String).includes("/admin/apis"));
  ok(!hrefs.map(String).includes("/admin/devices/activations"));
  ok(!hrefs.map(String).includes("/admin/devices/instances"));
  ok(!hrefs.map(String).includes("/admin/devices/reviews"));
  ok(hrefs.includes("/admin/portals"));
});

Deno.test("control panel exposes only routes backed by exact capabilities", () => {
  const profile = {
    active: true,
    capabilities: [`${apis.jobs.API.identity}::read`],
    email: null,
    id: "operator-1",
    name: "Job Reader",
    origin: "local",
  };
  const labels = getVisibleNavSections(profile).flatMap((section) =>
    section.items.map((item) => item.label)
  );

  deepEqual(labels, ["Account", "Jobs"]);
  ok(canAccessRoute("/admin/jobs", profile));
  ok(!canAccessRoute("/admin/events", profile));
});

Deno.test("control panel titles cover new admin routes", () => {
  deepEqual(getPageTitle("/admin/services"), "Services");
  deepEqual(getPageTitle("/admin/devices"), "Devices");
  deepEqual(getPageTitle("/admin/jobs"), "Jobs");
});
