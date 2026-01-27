import { NodeStatus, AgentStatus, TaskStatus, SessionStatus } from '../generated/dac/types/index.js';

/**
 * Get the string name of a status enum value
 */
function getStatusName<T extends Record<string | number, string | number>>(
  statusEnum: T,
  status: T[keyof T]
): string {
  return statusEnum[status]?.toString() || 'Unknown';
}

export const getNodeStatusName = (status: NodeStatus): string => 
  getStatusName(NodeStatus, status);

export const getAgentStatusName = (status: AgentStatus): string => 
  getStatusName(AgentStatus, status);

export const getTaskStatusName = (status: TaskStatus): string => 
  getStatusName(TaskStatus, status);

export const getSessionStatusName = (status: SessionStatus): string => 
  getStatusName(SessionStatus, status);
