import assert from "node:assert/strict";
import test from "node:test";

import {
  type ConnectionState,
  extensionHostEvidenceEnabled,
  statusPresentation,
} from "../../src/status";

test("maps every connection state to a fixed redacted presentation", () => {
  const states: readonly ConnectionState[] = [
    "disconnected",
    "connecting",
    "connected",
    "disconnecting",
    "failed",
  ];

  assert.deepEqual(
    states.map((state) => [state, statusPresentation(state)]),
    [
      [
        "disconnected",
        {
          text: "$(circle-outline) OneAgent",
          tooltip: "OneAgent is disconnected",
          command: "oneagent.connect",
        },
      ],
      [
        "connecting",
        { text: "$(sync~spin) OneAgent", tooltip: "OneAgent is connecting" },
      ],
      [
        "connected",
        {
          text: "$(check) OneAgent",
          tooltip: "OneAgent is connected",
          command: "oneagent.disconnect",
        },
      ],
      [
        "disconnecting",
        {
          text: "$(sync~spin) OneAgent",
          tooltip: "OneAgent is disconnecting",
        },
      ],
      [
        "failed",
        {
          text: "$(error) OneAgent",
          tooltip: "OneAgent connection failed",
          command: "oneagent.connect",
        },
      ],
    ],
  );
});

test("exposes Host evidence only for the two non-production test profiles", () => {
  for (const hostCase of ["trusted", "trusted-repeat"] as const) {
    assert.equal(extensionHostEvidenceEnabled(false, hostCase), true);
    assert.equal(extensionHostEvidenceEnabled(true, hostCase), false);
  }
  for (const hostCase of [undefined, "", "untrusted", "empty", "virtual", "multi-root"]) {
    assert.equal(extensionHostEvidenceEnabled(false, hostCase), false);
    assert.equal(extensionHostEvidenceEnabled(true, hostCase), false);
  }
});
