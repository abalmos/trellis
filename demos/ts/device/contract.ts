import { defineDeviceContract, state } from "@qlever-llc/trellis";
import {
  AssignmentsList,
  AuditRecorded,
  EvidenceDownload,
  EvidenceList,
  EvidenceUpload,
  EvidenceUploaded,
  ReportsGenerate,
  ReportsPublished,
  SitesGet,
  SitesList,
  SitesRefresh,
  SitesRefreshed,
} from "@trellis/apis/trellis.demo-service";
import { Type } from "typebox";

const schemas = {
  SelectedSiteState: Type.Object({
    siteId: Type.String(),
    siteName: Type.String(),
    selectedAt: Type.String({ format: "date-time" }),
  }),
  DraftInspectionState: Type.Object({
    inspectionId: Type.String(),
    siteId: Type.String(),
    checklistName: Type.String(),
    notes: Type.String(),
    updatedAt: Type.String({ format: "date-time" }),
  }),
} as const;

const contract = defineDeviceContract(
  { schemas },
  (ref) => ({
    id: "trellis.demo-device@v1",
    apiId: "trellis.demo-device@v1",
    apiVersion: "1.0.0",
    displayName: "Field Device Demo",
    description: "Activated Field Device TUI for the consolidated demo.",
    docs: {
      summary: "Activated field device demo.",
      markdown:
        "Declares the Field Device demo's service usage and local state for selected sites and draft inspections.",
    },
    uses: [
      AssignmentsList,
      SitesList,
      SitesGet,
      EvidenceList,
      EvidenceDownload,
      SitesRefresh,
      ReportsGenerate,
      EvidenceUpload,
      AuditRecorded.subscribe,
      ReportsPublished.subscribe,
      EvidenceUploaded.subscribe,
      SitesRefreshed.subscribe,
      state({
        selectedSite: {
          kind: "value",
          schema: ref.schema("SelectedSiteState"),
          stateVersion: "selected-site.v1",
          docs: {
            summary: "Selected site state.",
            markdown: "Stores the active site selected in the device TUI.",
          },
        },
        draftInspections: {
          kind: "map",
          schema: ref.schema("DraftInspectionState"),
          stateVersion: "draft-inspection.v1",
          docs: {
            summary: "Draft inspection state.",
            markdown:
              "Stores editable inspection draft notes keyed by inspection id.",
          },
        },
      }),
    ],
  }),
);

export default contract;
