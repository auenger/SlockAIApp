/**
 * Hook for Skill management in AgentsZone.
 *
 * Provides listing, adding, updating, and deleting Skills
 * for a specific Agent via the Rust backend.
 */

import { useState, useCallback } from "react";
import type { SkillInfo, SkillType, SkillStatus } from "../types";
import { listSkills, addSkill, updateSkill, deleteSkill } from "./ipc";

// ---------------------------------------------------------------------------
// Dev fallback: mock data when not running inside Tauri
// ---------------------------------------------------------------------------

const isTauri = "__TAURI_INTERNALS__" in window;

const MOCK_SKILLS: SkillInfo[] = [
  {
    id: "skill_1",
    agent_id: "default",
    name: "Terminal Access",
    skill_type: "Tool",
    config: { shell: "/bin/bash" },
    status: "Active",
    created_at: "2026-04-09T12:00:00Z",
    updated_at: "2026-04-09T12:00:00Z",
  },
  {
    id: "skill_2",
    agent_id: "default",
    name: "Web Search",
    skill_type: "MCP Server",
    config: { url: "http://localhost:3000/mcp" },
    status: "Active",
    created_at: "2026-04-09T12:00:00Z",
    updated_at: "2026-04-09T12:00:00Z",
  },
  {
    id: "skill_3",
    agent_id: "default",
    name: "Code Analysis",
    skill_type: "Tool",
    config: { languages: ["typescript", "rust"] },
    status: "Active",
    created_at: "2026-04-09T12:00:00Z",
    updated_at: "2026-04-09T12:00:00Z",
  },
  {
    id: "skill_4",
    agent_id: "default",
    name: "Database Query",
    skill_type: "MCP Server",
    config: { url: "http://localhost:5432" },
    status: "Inactive",
    created_at: "2026-04-09T12:00:00Z",
    updated_at: "2026-04-09T12:00:00Z",
  },
  {
    id: "skill_5",
    agent_id: "default",
    name: "Knowledge Base",
    skill_type: "Custom Command",
    config: { path: "/docs" },
    status: "Active",
    created_at: "2026-04-09T12:00:00Z",
    updated_at: "2026-04-09T12:00:00Z",
  },
];

// ---------------------------------------------------------------------------
// Hook return type
// ---------------------------------------------------------------------------

export interface SkillsState {
  /** List of skills for the current agent */
  skills: SkillInfo[];
  /** Loading state */
  loading: boolean;
  /** Error message if any */
  error: string | null;
  /** Load all skills for an agent */
  loadSkills: (agentId: string) => Promise<void>;
  /** Add a new skill */
  add: (agentId: string, name: string, skillType: SkillType, config: Record<string, unknown>) => Promise<void>;
  /** Update an existing skill */
  update: (agentId: string, skillId: string, updates: { name?: string; skill_type?: string; config?: Record<string, unknown>; status?: SkillStatus }) => Promise<void>;
  /** Delete a skill */
  remove: (agentId: string, skillId: string) => Promise<void>;
  /** Clear error */
  clearError: () => void;
}

// ---------------------------------------------------------------------------
// Hook implementation
// ---------------------------------------------------------------------------

export function useSkills(): SkillsState {
  const [skills, setSkills] = useState<SkillInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /** Load all skills for an agent */
  const loadSkills = useCallback(async (agentId: string) => {
    setLoading(true);
    setError(null);
    try {
      if (!isTauri) {
        setSkills(MOCK_SKILLS.filter((s) => s.agent_id === agentId));
        return;
      }
      const result = await listSkills(agentId);
      setSkills(result);
    } catch (err) {
      console.error("[useSkills] loadSkills failed:", err);
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  /** Add a new skill */
  const add = useCallback(async (agentId: string, name: string, skillType: SkillType, config: Record<string, unknown>) => {
    setError(null);
    try {
      if (!isTauri) {
        const newSkill: SkillInfo = {
          id: `skill_${Date.now()}`,
          agent_id: agentId,
          name,
          skill_type: skillType,
          config,
          status: "Active",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        };
        setSkills((prev) => [...prev, newSkill]);
        return;
      }
      await addSkill(agentId, name, skillType, config);
      await loadSkills(agentId);
    } catch (err) {
      console.error("[useSkills] add failed:", err);
      setError(String(err));
      throw err;
    }
  }, [loadSkills]);

  /** Update an existing skill */
  const update = useCallback(async (agentId: string, skillId: string, updates: { name?: string; skill_type?: string; config?: Record<string, unknown>; status?: SkillStatus }) => {
    setError(null);
    try {
      if (!isTauri) {
        setSkills((prev) =>
          prev.map((s) =>
            s.id === skillId
              ? {
                  ...s,
                  ...(updates.name ? { name: updates.name } : {}),
                  ...(updates.skill_type ? { skill_type: updates.skill_type as SkillType } : {}),
                  ...(updates.config ? { config: updates.config } : {}),
                  ...(updates.status ? { status: updates.status } : {}),
                  updated_at: new Date().toISOString(),
                }
              : s
          )
        );
        return;
      }
      await updateSkill(agentId, skillId, updates);
      await loadSkills(agentId);
    } catch (err) {
      console.error("[useSkills] update failed:", err);
      setError(String(err));
      throw err;
    }
  }, [loadSkills]);

  /** Delete a skill */
  const remove = useCallback(async (agentId: string, skillId: string) => {
    setError(null);
    try {
      if (!isTauri) {
        setSkills((prev) => prev.filter((s) => s.id !== skillId));
        return;
      }
      await deleteSkill(agentId, skillId);
      await loadSkills(agentId);
    } catch (err) {
      console.error("[useSkills] remove failed:", err);
      setError(String(err));
      throw err;
    }
  }, [loadSkills]);

  /** Clear error state */
  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    skills,
    loading,
    error,
    loadSkills,
    add,
    update,
    remove,
    clearError,
  };
}
