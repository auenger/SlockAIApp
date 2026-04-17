/**
 * Type-safe Tauri IPC wrapper.
 *
 * Provides invoke and listen helpers with proper TypeScript typing
 * for communication between the React frontend and Rust backend.
 */

import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import type {
  AgentSummary,
  CreateAgentRequest,
  UpdateAgentRequest,
  ManagerStatus,
  InitWorkspaceResult,
  IdentitySummary,
  AgentContextResult,
  AgentWithRuntime,
  AgentRuntimeInfo,
  RuntimeType,
  Thread,
  ThreadInfo,
  ThreadMessageData,
  Channel,
  ChannelInfo,
  DirectoryEntry,
  FileContent,
  ApiKeyInfo,
  SkillInfo,
  ActivityLogEntry,
  ListActivitiesResult,
  Task,
  TaskHistoryEntry,
  TaskDependency,
  CreateTaskInput,
  UpdateTaskInput,
  SuggestedTask,
  RemoteConnectionInfo,
  CreateRemoteConnectionRequest,
  UpdateRemoteConnectionRequest,
  TestConnectionResult,
  RemoteAgentCard,
  DelegationInfo,
  CreateDelegationRequest,
  ArtifactInfo,
  ArtifactContentResult,
  PushNotificationConfigInfo,
} from "../types";

/**
 * Type-safe invoke wrapper for Tauri commands.
 */
export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return tauriInvoke<T>(cmd, args);
}

// ---------------------------------------------------------------------------
// Test command
// ---------------------------------------------------------------------------

/** Send a greeting to the Rust backend (test command). */
export async function greet(name: string): Promise<string> {
  return invoke<string>("greet", { name });
}

// ---------------------------------------------------------------------------
// Runtime commands
// ---------------------------------------------------------------------------

/** Scan for available agent runtimes. */
export async function scanAgentRuntimes(): Promise<AgentRuntimeInfo[]> {
  return invoke<AgentRuntimeInfo[]>("scan_agent_runtimes");
}

/** List all registered runtimes using cached detection data. */
export async function listAgentRuntimes(): Promise<AgentRuntimeInfo[]> {
  return invoke<AgentRuntimeInfo[]>("list_agent_runtimes");
}

/** Get detailed info about a specific runtime by type. */
export async function getRuntimeInfo(runtimeType: RuntimeType): Promise<AgentRuntimeInfo> {
  return invoke<AgentRuntimeInfo>("get_runtime_info", { runtimeType });
}

// ---------------------------------------------------------------------------
// Workspace commands
// ---------------------------------------------------------------------------

/** Initialize the workspace for the first time. */
export async function initWorkspace(): Promise<InitWorkspaceResult> {
  return invoke<InitWorkspaceResult>("init_workspace");
}

/** Get workspace and agent manager status. */
export async function getWorkspaceStatus(): Promise<ManagerStatus> {
  return invoke<ManagerStatus>("get_workspace_status");
}

/** Health check result for the workspace. */
export interface HealthCheckResult {
  agents: import("../types").AgentHealthInfo[];
  repaired: number;
  still_unhealthy: number;
}

/** Perform a workspace health check and repair. */
export async function healthCheckWorkspace(): Promise<HealthCheckResult> {
  return invoke<HealthCheckResult>("health_check_workspace");
}

// ---------------------------------------------------------------------------
// Agent CRUD commands
// ---------------------------------------------------------------------------

/** Create a new Agent. */
export async function createAgent(request: CreateAgentRequest): Promise<AgentSummary> {
  return invoke<AgentSummary>("create_agent", { request });
}

/** List all available Agents. */
export async function listAgents(): Promise<AgentSummary[]> {
  return invoke<AgentSummary[]>("list_agents");
}

/** Switch to a different Agent. */
export async function switchAgent(agentId: string): Promise<AgentSummary> {
  return invoke<AgentSummary>("switch_agent", { agentId });
}

/** Get the currently active Agent. */
export async function getActiveAgent(): Promise<AgentSummary | null> {
  return invoke<AgentSummary | null>("get_active_agent");
}

/** Delete an Agent by ID. */
export async function deleteAgent(agentId: string): Promise<void> {
  return invoke<void>("delete_agent", { agentId });
}

/** Update an existing Agent's mutable properties. */
export async function updateAgent(
  agentId: string,
  request: UpdateAgentRequest
): Promise<AgentSummary> {
  return invoke<AgentSummary>("update_agent", { agentId, request });
}

// ---------------------------------------------------------------------------
// Identity commands
// ---------------------------------------------------------------------------

/** Get the identity of a specific Agent. */
export async function getAgentIdentity(agentId: string): Promise<IdentitySummary> {
  return invoke<IdentitySummary>("get_agent_identity", { agentId });
}

// ---------------------------------------------------------------------------
// Context commands
// ---------------------------------------------------------------------------

/** Get the context information for an Agent. */
export async function getAgentContext(agentId: string): Promise<AgentContextResult> {
  return invoke<AgentContextResult>("get_agent_context", { agentId });
}

// ---------------------------------------------------------------------------
// Agent + Runtime status commands
// ---------------------------------------------------------------------------

/** Get all agents fused with their runtime availability status. */
export async function getAgentRuntimeStatus(): Promise<AgentWithRuntime[]> {
  return invoke<AgentWithRuntime[]>("get_agent_runtime_status");
}

// ---------------------------------------------------------------------------
// Workspace browsing commands
// ---------------------------------------------------------------------------

/** List directory entries in an agent's workspace. */
export async function listWorkspaceDir(
  agentId: string,
  subpath?: string
): Promise<DirectoryEntry[]> {
  return invoke<DirectoryEntry[]>("list_workspace_dir", { agentId, subpath: subpath ?? null });
}

/** Read the content of a file from an agent's workspace. */
export async function readWorkspaceFile(
  agentId: string,
  filePath: string
): Promise<FileContent> {
  return invoke<FileContent>("read_workspace_file", { agentId, filePath });
}

/** Open an agent's workspace directory in the system file manager (Finder/Explorer). */
export async function openWorkspaceInFinder(agentId: string): Promise<void> {
  return invoke<void>("open_in_finder", { agentId });
}

// ---------------------------------------------------------------------------
// Thread commands
// ---------------------------------------------------------------------------

/** Create a new Thread for a specific agent. */
export async function createThread(agentId: string): Promise<Thread> {
  return invoke<Thread>("create_thread", { agentId });
}

/** List all threads for a specific agent. */
export async function listThreads(agentId: string): Promise<ThreadInfo[]> {
  return invoke<ThreadInfo[]>("list_threads", { agentId });
}

/** Get a single thread by ID. */
export async function getThread(agentId: string, threadId: string): Promise<Thread> {
  return invoke<Thread>("get_thread", { agentId, threadId });
}

/** Delete a thread by ID. */
export async function deleteThread(agentId: string, threadId: string): Promise<void> {
  return invoke<void>("delete_thread", { agentId, threadId });
}

/** Send a message in a thread (triggers runtime streaming). */
export async function sendMessage(
  agentId: string,
  threadId: string,
  message: string
): Promise<Thread> {
  return invoke<Thread>("send_message", { agentId, threadId, message });
}

/** Save an agent response to a thread (after streaming completes). */
export async function saveAgentResponse(
  agentId: string,
  threadId: string,
  content: string,
  sessionId: string | null
): Promise<Thread> {
  return invoke<Thread>("save_agent_response", { agentId, threadId, content, sessionId });
}

/** Load messages for a thread from JSONL storage (for crash recovery). */
export async function loadThreadMessages(
  agentId: string,
  threadId: string,
  limit?: number
): Promise<ThreadMessageData[]> {
  return invoke<ThreadMessageData[]>("load_thread_messages", { agentId, threadId, limit: limit ?? null });
}

/** List all threads across all agents (global thread list). */
export async function listAllThreads(): Promise<ThreadInfo[]> {
  return invoke<ThreadInfo[]>("list_all_threads");
}

/** Rename a thread by updating its title. */
export async function renameThread(
  threadId: string,
  newTitle: string
): Promise<ThreadInfo> {
  return invoke<ThreadInfo>("rename_thread", { threadId, newTitle });
}

// ---------------------------------------------------------------------------
// Channel commands
// ---------------------------------------------------------------------------

/** Create a new Channel with the given name and Agent members. */
export async function createChannel(
  name: string,
  memberAgentIds: string[]
): Promise<Channel> {
  return invoke<Channel>("create_channel", {
    request: { name, member_agent_ids: memberAgentIds },
  });
}

/** List all channels. */
export async function listChannels(): Promise<ChannelInfo[]> {
  return invoke<ChannelInfo[]>("list_channels");
}

/** Get a single channel by ID (with full details). */
export async function getChannel(channelId: string): Promise<Channel> {
  return invoke<Channel>("get_channel", { channelId });
}

/** Update a channel's settings. */
export async function updateChannel(
  channelId: string,
  name?: string
): Promise<Channel> {
  return invoke<Channel>("update_channel", { channelId, request: { name } });
}

/** Delete a channel by ID. */
export async function deleteChannel(channelId: string): Promise<void> {
  return invoke<void>("delete_channel", { channelId });
}

/** Add an Agent member to a channel. */
export async function addChannelMember(
  channelId: string,
  agentId: string
): Promise<Channel> {
  return invoke<Channel>("add_channel_member", { channelId, agentId });
}

/** Remove an Agent member from a channel. */
export async function removeChannelMember(
  channelId: string,
  agentId: string
): Promise<Channel> {
  return invoke<Channel>("remove_channel_member", { channelId, agentId });
}

/** Send a message in a channel (triggers runtime streaming). */
export async function sendChannelMessage(
  channelId: string,
  message: string,
  userName?: string,
): Promise<import("../types").SendChannelMessageResponse> {
  return invoke<import("../types").SendChannelMessageResponse>("send_channel_message", { channelId, message, userName });
}

/** Save an agent response to a channel (after streaming completes). */
export async function saveChannelResponse(
  channelId: string,
  agentId: string,
  content: string
): Promise<Channel> {
  return invoke<Channel>("save_channel_response", { channelId, agentId, content });
}

/** Compact (summarize) older messages in a channel. */
export async function compactChannel(
  channelId: string
): Promise<Channel> {
  return invoke<Channel>("compact_channel", { channelId });
}

// ---------------------------------------------------------------------------
// API Key management commands
// ---------------------------------------------------------------------------

/** List all API keys (masked) for known providers. */
export async function listApiKeys(): Promise<ApiKeyInfo[]> {
  return invoke<ApiKeyInfo[]>("list_api_keys");
}

/** Store a new API key for a provider. */
export async function storeApiKey(runtimeId: string, apiKey: string): Promise<void> {
  return invoke<void>("store_api_key", { runtimeId, apiKey });
}

/** Check if an API key exists for a provider. */
export async function hasApiKey(runtimeId: string): Promise<boolean> {
  return invoke<boolean>("has_api_key", { runtimeId });
}

/** Delete an API key for a provider. */
export async function deleteApiKey(runtimeId: string): Promise<void> {
  return invoke<void>("delete_api_key", { runtimeId });
}

/** Verify an API key and return its masked info. */
export async function verifyApiKey(runtimeId: string): Promise<ApiKeyInfo> {
  return invoke<ApiKeyInfo>("verify_api_key", { runtimeId });
}

// ---------------------------------------------------------------------------
// Skill management commands
// ---------------------------------------------------------------------------

/** List all Skills for a given Agent. */
export async function listSkills(agentId: string): Promise<SkillInfo[]> {
  return invoke<SkillInfo[]>("list_skills", { agentId });
}

/** Add a new Skill to an Agent. */
export async function addSkill(
  agentId: string,
  name: string,
  skillType: string,
  config: Record<string, unknown>
): Promise<SkillInfo> {
  return invoke<SkillInfo>("add_skill", {
    agentId,
    request: { name, skill_type: skillType, config },
  });
}

/** Update an existing Skill. */
export async function updateSkill(
  agentId: string,
  skillId: string,
  request: {
    name?: string;
    skill_type?: string;
    config?: Record<string, unknown>;
    status?: string;
  }
): Promise<SkillInfo> {
  return invoke<SkillInfo>("update_skill", { agentId, skillId, request });
}

/** Delete a Skill. */
export async function deleteSkill(
  agentId: string,
  skillId: string
): Promise<void> {
  return invoke<void>("delete_skill", { agentId, skillId });
}

/** Get the status of a single Skill. */
export async function getSkillStatus(
  agentId: string,
  skillId: string
): Promise<SkillInfo> {
  return invoke<SkillInfo>("get_skill_status", { agentId, skillId });
}

// ---------------------------------------------------------------------------
// Activity Log commands
// ---------------------------------------------------------------------------

/** Log a new activity entry. */
export async function logActivity(params: {
  activity_type: string;
  agent_id?: string | null;
  summary: string;
  details?: Record<string, unknown>;
}): Promise<ActivityLogEntry> {
  return invoke<ActivityLogEntry>("log_activity", {
    request: {
      activity_type: params.activity_type,
      agent_id: params.agent_id ?? null,
      summary: params.summary,
      details: params.details ?? {},
    },
  });
}

/** List activity entries with optional filter and pagination. */
export async function listActivities(params?: {
  agent_id?: string | null;
  offset?: number;
  limit?: number;
}): Promise<ListActivitiesResult> {
  return invoke<ListActivitiesResult>("list_activities", {
    request: {
      agent_id: params?.agent_id ?? null,
      offset: params?.offset ?? 0,
      limit: params?.limit ?? 50,
    },
  });
}

/** Clear all activity entries. */
export async function clearActivities(): Promise<void> {
  return invoke<void>("clear_activities");
}

// ---------------------------------------------------------------------------
// Task commands
// ---------------------------------------------------------------------------

/** Create a new Task. */
export async function createTask(input: CreateTaskInput): Promise<Task> {
  return invoke<Task>("create_task", {
    request: {
      title: input.title,
      description: input.description ?? "",
      priority: input.priority ?? 3,
      creator_id: input.creatorId,
      creator_type: input.creatorType ?? "user",
      assignee_id: input.assigneeId ?? null,
      channel_id: input.channelId ?? null,
      thread_id: input.threadId ?? null,
      parent_task_id: input.parentTaskId ?? null,
      execution_mode: input.executionMode ?? "realtime",
      source: input.source ?? "manual",
      source_message_id: input.sourceMessageId ?? null,
    },
  });
}

/** List tasks with optional filters. */
export async function listTasks(params?: {
  statusFilter?: string;
  channelId?: string;
  assigneeId?: string;
  parentTaskId?: string;
}): Promise<Task[]> {
  return invoke<Task[]>("list_tasks", {
    statusFilter: params?.statusFilter ?? null,
    channelId: params?.channelId ?? null,
    assigneeId: params?.assigneeId ?? null,
    parentTaskId: params?.parentTaskId ?? null,
  });
}

/** Get a single task by ID. */
export async function getTask(taskId: string): Promise<Task> {
  return invoke<Task>("get_task", { taskId });
}

/** Update an existing task. */
export async function updateTask(
  taskId: string,
  input: UpdateTaskInput
): Promise<Task> {
  return invoke<Task>("update_task", {
    taskId,
    request: {
      title: input.title ?? null,
      description: input.description ?? null,
      status: input.status ?? null,
      priority: input.priority ?? null,
      assignee_id: input.assigneeId !== undefined ? input.assigneeId : null,
      execution_mode: input.executionMode ?? null,
      result: input.result !== undefined ? input.result : null,
    },
  });
}

/** Delete a task by ID. */
export async function deleteTask(taskId: string): Promise<void> {
  return invoke<void>("delete_task", { taskId });
}

/** Update only the status of a task (for drag-and-drop). */
export async function updateTaskStatus(
  taskId: string,
  status: string
): Promise<Task> {
  return invoke<Task>("update_task_status", { taskId, status });
}

/** Assign or reassign a task to an agent. */
export async function assignTask(
  taskId: string,
  agentId: string | null
): Promise<Task> {
  return invoke<Task>("assign_task", { taskId, agentId });
}

/** Cancel a task (sets status to 'cancelled'). */
export async function cancelTask(taskId: string): Promise<Task> {
  return invoke<Task>("cancel_task", { taskId });
}

/** Add a dependency: task_id depends on depends_on_id. */
export async function addTaskDependency(
  taskId: string,
  dependsOnId: string
): Promise<void> {
  return invoke<void>("add_task_dependency", { taskId, dependsOnId });
}

/** Remove a dependency. */
export async function removeTaskDependency(
  taskId: string,
  dependsOnId: string
): Promise<void> {
  return invoke<void>("remove_task_dependency", { taskId, dependsOnId });
}

/** Get the history entries for a task. */
export async function getTaskHistory(
  taskId: string
): Promise<TaskHistoryEntry[]> {
  return invoke<TaskHistoryEntry[]>("get_task_history", { taskId });
}

/** Get dependencies for a task (tasks that this task depends on). */
export async function getTaskDependencies(
  taskId: string
): Promise<TaskDependency[]> {
  return invoke<TaskDependency[]>("get_task_dependencies", { taskId });
}

/** Get tasks that depend on a given task (reverse dependencies). */
export async function getDependentTasks(
  taskId: string
): Promise<TaskDependency[]> {
  return invoke<TaskDependency[]>("get_dependent_tasks", { taskId });
}

/** Get child tasks of a parent task. */
export async function getChildTasks(
  parentTaskId: string
): Promise<Task[]> {
  return invoke<Task[]>("get_child_tasks", { parentTaskId });
}

// ---------------------------------------------------------------------------
// Task Engine execution commands
// ---------------------------------------------------------------------------

/** Execute a task via the TaskEngine (realtime or async based on task's execution_mode). */
export async function executeTask(taskId: string): Promise<void> {
  return invoke<void>("execute_task", { taskId });
}

/** Cancel a running task execution via the TaskEngine. */
export async function cancelTaskExecution(taskId: string): Promise<void> {
  return invoke<void>("cancel_task_execution", { taskId });
}

/** Report task completion to the TaskEngine (called by frontend after agent responds). */
export async function reportTaskCompleted(
  taskId: string,
  result: string
): Promise<void> {
  return invoke<void>("report_task_completed", { taskId, result });
}

/** Report task failure to the TaskEngine. */
export async function reportTaskFailed(
  taskId: string,
  error: string
): Promise<void> {
  return invoke<void>("report_task_failed", { taskId, error });
}

/** Get active task execution status from the TaskEngine. */
export async function getTaskEngineStatus(): Promise<{
  active_tasks: Array<{
    task_id: string;
    agent_id: string;
    channel_id: string;
    started_at: string;
    mode: string;
  }>;
  queue_length: number;
}> {
  return invoke("get_task_engine_status");
}

// ---------------------------------------------------------------------------
// Task Suggestion commands
// ---------------------------------------------------------------------------

/** Confirm task suggestions — creates Tasks from the selected suggestions. */
export async function confirmTaskSuggestions(
  messageId: string,
  channelId: string,
  selected: SuggestedTask[],
  agentId?: string,
  source?: string
): Promise<Task[]> {
  return invoke<Task[]>("confirm_task_suggestions", {
    messageId,
    channelId,
    selected,
    agentId: agentId ?? null,
    source: source ?? null,
  });
}

/** Dismiss task suggestions — marks the suggestion message as dismissed. */
export async function dismissTaskSuggestions(
  messageId: string,
  channelId: string
): Promise<void> {
  return invoke<void>("dismiss_task_suggestions", { messageId, channelId });
}

// ---------------------------------------------------------------------------
// Remote Connection commands
// ---------------------------------------------------------------------------

/** Create a new remote A2A connection. */
export async function remoteConnectionCreate(
  request: CreateRemoteConnectionRequest
): Promise<RemoteConnectionInfo> {
  return invoke<RemoteConnectionInfo>("remote_connection_create", { request });
}

/** List all remote connections. */
export async function remoteConnectionList(): Promise<RemoteConnectionInfo[]> {
  return invoke<RemoteConnectionInfo[]>("remote_connection_list");
}

/** Update a remote connection. */
export async function remoteConnectionUpdate(
  id: string,
  request: UpdateRemoteConnectionRequest
): Promise<RemoteConnectionInfo> {
  return invoke<RemoteConnectionInfo>("remote_connection_update", { id, request });
}

/** Delete a remote connection. */
export async function remoteConnectionDelete(id: string): Promise<void> {
  return invoke<void>("remote_connection_delete", { id });
}

/** Test a remote connection (health check). */
export async function remoteConnectionTest(
  id: string
): Promise<TestConnectionResult> {
  return invoke<TestConnectionResult>("remote_connection_test", { id });
}

/** Batch health check all remote connections. */
export async function remoteConnectionHealthAll(): Promise<RemoteConnectionInfo[]> {
  return invoke<RemoteConnectionInfo[]>("remote_connection_health_all");
}

/** Get the cached AgentCard for a remote connection. */
export async function remoteConnectionGetAgentCard(
  id: string
): Promise<RemoteAgentCard | null> {
  return invoke<RemoteAgentCard | null>("remote_connection_get_agent_card", { id });
}

// ---------------------------------------------------------------------------
// Remote Agent Sync commands
// ---------------------------------------------------------------------------

/** Sync remote agents from a specific connection. */
export async function syncRemoteAgents(
  connectionId: string
): Promise<AgentSummary[]> {
  return invoke<AgentSummary[]>("sync_remote_agents", { connectionId });
}

/** Get all remote agents across all connections. */
export async function getRemoteAgents(): Promise<AgentSummary[]> {
  return invoke<AgentSummary[]>("get_remote_agents");
}

/** Refresh agents for a specific connection (health check + sync). */
export async function refreshRemoteAgents(
  connectionId: string
): Promise<void> {
  return invoke<void>("refresh_remote_agents", { connectionId });
}

// ---------------------------------------------------------------------------
// Collaboration / A2A Multi-Agent commands
// ---------------------------------------------------------------------------

/** Create a delegation from one agent to another. */
export async function collaborationDelegate(
  request: CreateDelegationRequest
): Promise<DelegationInfo> {
  return invoke<DelegationInfo>("collaboration_delegate", { request });
}

/** List delegations with optional filters. */
export async function collaborationListDelegations(params?: {
  agentId?: string;
  activeOnly?: boolean;
}): Promise<DelegationInfo[]> {
  return invoke<DelegationInfo[]>("collaboration_list_delegations", {
    agentId: params?.agentId ?? null,
    activeOnly: params?.activeOnly ?? false,
  });
}

/** Cancel a delegation. */
export async function collaborationCancelDelegation(
  delegationId: string
): Promise<DelegationInfo> {
  return invoke<DelegationInfo>("collaboration_cancel_delegation", { delegationId });
}

/** Retry a failed delegation. */
export async function collaborationRetryDelegation(
  delegationId: string
): Promise<DelegationInfo> {
  return invoke<DelegationInfo>("collaboration_retry_delegation", { delegationId });
}

/** List artifacts with optional filters. */
export async function collaborationListArtifacts(params?: {
  agentId?: string;
  taskId?: string;
}): Promise<ArtifactInfo[]> {
  return invoke<ArtifactInfo[]>("collaboration_list_artifacts", {
    agentId: params?.agentId ?? null,
    taskId: params?.taskId ?? null,
  });
}

/** Get artifact content. */
export async function collaborationGetArtifact(
  artifactId: string,
  consumerAgentId?: string
): Promise<ArtifactContentResult> {
  return invoke<ArtifactContentResult>("collaboration_get_artifact", {
    artifactId,
    consumerAgentId: consumerAgentId ?? null,
  });
}

/** Search artifacts by name. */
export async function collaborationSearchArtifacts(
  query: string
): Promise<ArtifactInfo[]> {
  return invoke<ArtifactInfo[]>("collaboration_search_artifacts", { query });
}

/** Register a new artifact. */
export async function collaborationRegisterArtifact(params: {
  producerAgentId: string;
  name: string;
  filePath: string;
  mimeType?: string;
  taskId?: string;
  description?: string;
}): Promise<ArtifactInfo> {
  return invoke<ArtifactInfo>("collaboration_register_artifact", {
    producerAgentId: params.producerAgentId,
    name: params.name,
    filePath: params.filePath,
    mimeType: params.mimeType ?? null,
    taskId: params.taskId ?? null,
    description: params.description ?? null,
  });
}

/** Register a push notification endpoint. */
export async function collaborationRegisterPushUrl(params: {
  url: string;
  token?: string;
  hmacSecret?: string;
  events?: string[];
}): Promise<PushNotificationConfigInfo> {
  return invoke<PushNotificationConfigInfo>("collaboration_register_push_url", {
    url: params.url,
    token: params.token ?? null,
    hmacSecret: params.hmacSecret ?? null,
    events: params.events ?? null,
  });
}

/** List push notification configs. */
export async function collaborationListPushConfigs(): Promise<PushNotificationConfigInfo[]> {
  return invoke<PushNotificationConfigInfo[]>("collaboration_list_push_configs");
}

/** Unregister a push notification config. */
export async function collaborationUnregisterPushUrl(
  configId: string
): Promise<boolean> {
  return invoke<boolean>("collaboration_unregister_push_url", { configId });
}

// ---------------------------------------------------------------------------
// LAN A2A Server commands
// ---------------------------------------------------------------------------

import type { LanServerInfo } from "../types";

/** Start the A2A LAN server on the given port. */
export async function startA2aServer(port: number): Promise<LanServerInfo> {
  return invoke<LanServerInfo>("start_a2a_server", { port });
}

/** Stop the A2A LAN server. */
export async function stopA2aServer(): Promise<void> {
  return invoke<void>("stop_a2a_server");
}

/** Get the current status of the A2A LAN server. */
export async function getA2aServerStatus(): Promise<LanServerInfo> {
  return invoke<LanServerInfo>("get_a2a_server_status");
}

/** Get the local IP addresses of this machine. */
export async function getLocalIpAddresses(): Promise<string[]> {
  return invoke<string[]>("get_local_ip_addresses");
}
