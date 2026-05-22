/**
 * Spy-Code Integration Core Library
 * Main entry point for the shared integration library
 */

// Export types
export * from './types';

// Export CLI bridge
export { CLIBridge } from './cli-bridge';

// Export MCP adapter
export { MCPClient } from './mcp-adapter/client';

// Export agent hooks
export { AgentHooks, getAgentHooks, resetAgentHooks } from './agent-hooks/interface';

// Export cache manager
export { CacheManager } from './cache-manager';

// Export event bus
export { EventBus, eventBus, EventType, EventPayload } from './event-bus';

// Export skill engine
export { SkillLoader, SkillDefinition } from './skill-engine/skill-loader';
export { SkillMatcher, MatchResult } from './skill-engine/skill-matcher';
export { SkillExecutor, ExecutionContext, ExecutionResult } from './skill-engine/skill-executor';
export { SkillRegistry, getSkillRegistry } from './skill-engine/skill-registry';
