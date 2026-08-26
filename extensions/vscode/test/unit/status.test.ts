import assert from "node:assert/strict";
import test from "node:test";

import { type ConnectionState, statusPresentation } from "../../src/status";

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
