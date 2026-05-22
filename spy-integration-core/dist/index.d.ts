/**
 * Spy-Code Integration Core Library
 * Main entry point for the shared integration library
 */
export * from './types';
export { CLIBridge } from './cli-bridge';
export { MCPClient } from './mcp-adapter/client';
export { AgentHooks, getAgentHooks, resetAgentHooks } from './agent-hooks/interface';
export { CacheManager } from './cache-manager';
export { EventBus, eventBus, EventType, EventPayload } from './event-bus';
export { SkillLoader, SkillDefinition } from './skill-engine/skill-loader';
export { SkillMatcher, MatchResult } from './skill-engine/skill-matcher';
export { SkillExecutor, ExecutionContext, ExecutionResult } from './skill-engine/skill-executor';
export { SkillRegistry, getSkillRegistry } from './skill-engine/skill-registry';
