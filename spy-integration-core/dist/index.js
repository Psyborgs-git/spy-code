"use strict";
/**
 * Spy-Code Integration Core Library
 * Main entry point for the shared integration library
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.getSkillRegistry = exports.SkillRegistry = exports.SkillExecutor = exports.SkillMatcher = exports.SkillLoader = exports.EventType = exports.eventBus = exports.EventBus = exports.CacheManager = exports.resetAgentHooks = exports.getAgentHooks = exports.AgentHooks = exports.MCPClient = exports.CLIBridge = void 0;
// Export types
__exportStar(require("./types"), exports);
// Export CLI bridge
var cli_bridge_1 = require("./cli-bridge");
Object.defineProperty(exports, "CLIBridge", { enumerable: true, get: function () { return cli_bridge_1.CLIBridge; } });
// Export MCP adapter
var client_1 = require("./mcp-adapter/client");
Object.defineProperty(exports, "MCPClient", { enumerable: true, get: function () { return client_1.MCPClient; } });
// Export agent hooks
var interface_1 = require("./agent-hooks/interface");
Object.defineProperty(exports, "AgentHooks", { enumerable: true, get: function () { return interface_1.AgentHooks; } });
Object.defineProperty(exports, "getAgentHooks", { enumerable: true, get: function () { return interface_1.getAgentHooks; } });
Object.defineProperty(exports, "resetAgentHooks", { enumerable: true, get: function () { return interface_1.resetAgentHooks; } });
// Export cache manager
var cache_manager_1 = require("./cache-manager");
Object.defineProperty(exports, "CacheManager", { enumerable: true, get: function () { return cache_manager_1.CacheManager; } });
// Export event bus
var event_bus_1 = require("./event-bus");
Object.defineProperty(exports, "EventBus", { enumerable: true, get: function () { return event_bus_1.EventBus; } });
Object.defineProperty(exports, "eventBus", { enumerable: true, get: function () { return event_bus_1.eventBus; } });
Object.defineProperty(exports, "EventType", { enumerable: true, get: function () { return event_bus_1.EventType; } });
// Export skill engine
var skill_loader_1 = require("./skill-engine/skill-loader");
Object.defineProperty(exports, "SkillLoader", { enumerable: true, get: function () { return skill_loader_1.SkillLoader; } });
var skill_matcher_1 = require("./skill-engine/skill-matcher");
Object.defineProperty(exports, "SkillMatcher", { enumerable: true, get: function () { return skill_matcher_1.SkillMatcher; } });
var skill_executor_1 = require("./skill-engine/skill-executor");
Object.defineProperty(exports, "SkillExecutor", { enumerable: true, get: function () { return skill_executor_1.SkillExecutor; } });
var skill_registry_1 = require("./skill-engine/skill-registry");
Object.defineProperty(exports, "SkillRegistry", { enumerable: true, get: function () { return skill_registry_1.SkillRegistry; } });
Object.defineProperty(exports, "getSkillRegistry", { enumerable: true, get: function () { return skill_registry_1.getSkillRegistry; } });
