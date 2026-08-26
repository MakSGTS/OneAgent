export type ConnectionState =
  | "disconnected"
  | "connecting"
  | "connected"
  | "disconnecting"
  | "failed";

export interface StatusPresentation {
  readonly text: string;
  readonly tooltip: string;
  readonly command?: "oneagent.connect" | "oneagent.disconnect";
}

const PRESENTATIONS: Readonly<Record<ConnectionState, StatusPresentation>> = {
  disconnected: {
    text: "$(circle-outline) OneAgent",
    tooltip: "OneAgent is disconnected",
    command: "oneagent.connect",
  },
  connecting: {
    text: "$(sync~spin) OneAgent",
    tooltip: "OneAgent is connecting",
  },
  connected: {
    text: "$(check) OneAgent",
    tooltip: "OneAgent is connected",
    command: "oneagent.disconnect",
  },
  disconnecting: {
    text: "$(sync~spin) OneAgent",
    tooltip: "OneAgent is disconnecting",
  },
  failed: {
    text: "$(error) OneAgent",
    tooltip: "OneAgent connection failed",
    command: "oneagent.connect",
  },
};

export function statusPresentation(state: ConnectionState): StatusPresentation {
  return PRESENTATIONS[state];
}
