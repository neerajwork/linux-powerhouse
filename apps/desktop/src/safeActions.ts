export type SafeActionId =
  | "refresh_health"
  | "storage_diagnostic"
  | "process_diagnostic"
  | "network_diagnostic"
  | "service_diagnostic";

export type SafeActionDefinition = {
  id: SafeActionId;
  label: string;
  subsystem: "System" | "Storage" | "Processes" | "Network" | "Services";
  changesSystemState: boolean;
  reversible: boolean;
  privilege: "None";
};

/**
 * Deliberately narrow Step 41 allowlist. These actions refresh or inspect
 * health data only; they do not mutate operating-system state.
 */
export const SAFE_ACTIONS: SafeActionDefinition[] = [
  { id: "refresh_health", label: "Refresh health snapshot", subsystem: "System", changesSystemState: false, reversible: true, privilege: "None" },
  { id: "storage_diagnostic", label: "Run storage diagnostic", subsystem: "Storage", changesSystemState: false, reversible: true, privilege: "None" },
  { id: "process_diagnostic", label: "Run process diagnostic", subsystem: "Processes", changesSystemState: false, reversible: true, privilege: "None" },
  { id: "network_diagnostic", label: "Run network diagnostic", subsystem: "Network", changesSystemState: false, reversible: true, privilege: "None" },
  { id: "service_diagnostic", label: "Run service diagnostic", subsystem: "Services", changesSystemState: false, reversible: true, privilege: "None" },
];

export function actionForSubsystem(subsystem: SafeActionDefinition["subsystem"]): SafeActionDefinition {
  const action = SAFE_ACTIONS.find((candidate) => candidate.subsystem === subsystem);
  return action ?? SAFE_ACTIONS[0];
}
