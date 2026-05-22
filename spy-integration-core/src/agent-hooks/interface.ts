/**
 * Agent Hooks Interface - Coding agent lifecycle hooks
 */

import { EventEmitter } from 'eventemitter3';
import {
  HookType,
  HookContext,
  HookHandler,
  HookResult,
  CodeModification
} from '../types';

export class AgentHooks extends EventEmitter {
  private hooks: Map<HookType, Set<HookHandler>> = new Map();
  private enabled: boolean = true;

  /**
   * Register a hook handler
   */
  registerHook(hookType: HookType, handler: HookHandler): void {
    if (!this.hooks.has(hookType)) {
      this.hooks.set(hookType, new Set());
    }
    this.hooks.get(hookType)!.add(handler);
    this.emit('hook_registered', { hookType, handler });
  }

  /**
   * Unregister a hook handler
   */
  unregisterHook(hookType: HookType, handler: HookHandler): void {
    const handlers = this.hooks.get(hookType);
    if (handlers) {
      handlers.delete(handler);
      if (handlers.size === 0) {
        this.hooks.delete(hookType);
      }
      this.emit('hook_unregistered', { hookType, handler });
    }
  }

  /**
   * Execute all handlers for a hook type
   */
  async executeHook(hookType: HookType, context: HookContext): Promise<HookResult> {
    if (!this.enabled) {
      return { continue: true };
    }

    const handlers = this.hooks.get(hookType);
    if (!handlers || handlers.size === 0) {
      return { continue: true };
    }

    const result: HookResult = { continue: true };
    const modifications: CodeModification[] = [];
    const metadata: Record<string, any> = {};

    for (const handler of handlers) {
      try {
        const handlerResult = await handler(context);

        // If any handler says to stop, stop execution
        if (!handlerResult.continue) {
          result.continue = false;
        }

        // Collect modifications
        if (handlerResult.modifications) {
          modifications.push(...handlerResult.modifications);
        }

        // Collect metadata
        if (handlerResult.metadata) {
          Object.assign(metadata, handlerResult.metadata);
        }

        // If handler returned an error, propagate it
        if (handlerResult.error) {
          result.error = handlerResult.error;
          break;
        }

        this.emit('hook_executed', { hookType, handler, result: handlerResult });
      } catch (error) {
        result.error = error as Error;
        result.continue = false;
        this.emit('hook_error', { hookType, handler, error });
        break;
      }
    }

    if (modifications.length > 0) {
      result.modifications = modifications;
    }

    if (Object.keys(metadata).length > 0) {
      result.metadata = metadata;
    }

    return result;
  }

  /**
   * Enable hooks
   */
  enable(): void {
    this.enabled = true;
    this.emit('hooks_enabled');
  }

  /**
   * Disable hooks
   */
  disable(): void {
    this.enabled = false;
    this.emit('hooks_disabled');
  }

  /**
   * Check if hooks are enabled
   */
  isEnabled(): boolean {
    return this.enabled;
  }

  /**
   * Get all registered hooks
   */
  getRegisteredHooks(): Map<HookType, number> {
    const result = new Map<HookType, number>();
    for (const [type, handlers] of this.hooks.entries()) {
      result.set(type, handlers.size);
    }
    return result;
  }

  /**
   * Clear all hooks
   */
  clearAll(): void {
    this.hooks.clear();
    this.emit('hooks_cleared');
  }

  /**
   * Clear hooks for a specific type
   */
  clearType(hookType: HookType): void {
    this.hooks.delete(hookType);
    this.emit('hooks_cleared_type', { hookType });
  }

  /**
   * Pre-read code hook
   */
  async preReadCode(context: HookContext): Promise<HookResult> {
    return this.executeHook(HookType.PRE_READ_CODE, context);
  }

  /**
   * Post-read code hook
   */
  async postReadCode(context: HookContext): Promise<HookResult> {
    return this.executeHook(HookType.POST_READ_CODE, context);
  }

  /**
   * Pre-write code hook
   */
  async preWriteCode(context: HookContext): Promise<HookResult> {
    return this.executeHook(HookType.PRE_WRITE_CODE, context);
  }

  /**
   * Post-write code hook
   */
  async postWriteCode(context: HookContext): Promise<HookResult> {
    return this.executeHook(HookType.POST_WRITE_CODE, context);
  }

  /**
   * Pre-run command hook
   */
  async preRunCommand(context: HookContext): Promise<HookResult> {
    return this.executeHook(HookType.PRE_RUN_COMMAND, context);
  }

  /**
   * Post-run command hook
   */
  async postRunCommand(context: HookContext): Promise<HookResult> {
    return this.executeHook(HookType.POST_RUN_COMMAND, context);
  }

  /**
   * Pre-MCP tool use hook
   */
  async preMcpToolUse(context: HookContext): Promise<HookResult> {
    return this.executeHook(HookType.PRE_MCP_TOOL_USE, context);
  }

  /**
   * Post-MCP tool use hook
   */
  async postMcpToolUse(context: HookContext): Promise<HookResult> {
    return this.executeHook(HookType.POST_MCP_TOOL_USE, context);
  }

  /**
   * Pre-user prompt hook
   */
  async preUserPrompt(context: HookContext): Promise<HookResult> {
    return this.executeHook(HookType.PRE_USER_PROMPT, context);
  }

  /**
   * Post-cascade response hook
   */
  async postCascadeResponse(context: HookContext): Promise<HookResult> {
    return this.executeHook(HookType.POST_CASCADE_RESPONSE, context);
  }

  /**
   * Post-setup worktree hook
   */
  async postSetupWorktree(context: HookContext): Promise<HookResult> {
    return this.executeHook(HookType.POST_SETUP_WORKTREE, context);
  }
}

// Create singleton instance
let agentHooksInstance: AgentHooks | null = null;

export function getAgentHooks(): AgentHooks {
  if (!agentHooksInstance) {
    agentHooksInstance = new AgentHooks();
  }
  return agentHooksInstance;
}

export function resetAgentHooks(): void {
  if (agentHooksInstance) {
    agentHooksInstance.clearAll();
    agentHooksInstance = null;
  }
}
