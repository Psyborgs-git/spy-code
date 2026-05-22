"use strict";
/**
 * Core type definitions for spy-code integration
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.NotificationType = exports.CacheError = exports.MCPError = exports.CLIError = exports.SpyCodeError = exports.HookType = exports.EdgeKind = exports.Language = exports.NodeKind = void 0;
// Node types from spy-code schema
var NodeKind;
(function (NodeKind) {
    NodeKind["FUNCTION"] = "function";
    NodeKind["CLASS"] = "class";
    NodeKind["CONSTANT"] = "constant";
    NodeKind["DEPENDENCY"] = "dependency";
})(NodeKind || (exports.NodeKind = NodeKind = {}));
var Language;
(function (Language) {
    Language["RUST"] = "rust";
    Language["PYTHON"] = "python";
    Language["TYPESCRIPT"] = "typescript";
    Language["JAVASCRIPT"] = "javascript";
    Language["GO"] = "go";
})(Language || (exports.Language = Language = {}));
var EdgeKind;
(function (EdgeKind) {
    EdgeKind["CALLS"] = "calls";
    EdgeKind["IMPORTS"] = "imports";
    EdgeKind["REFERENCES"] = "references";
})(EdgeKind || (exports.EdgeKind = EdgeKind = {}));
// Hook types
var HookType;
(function (HookType) {
    HookType["PRE_READ_CODE"] = "pre_read_code";
    HookType["POST_READ_CODE"] = "post_read_code";
    HookType["PRE_WRITE_CODE"] = "pre_write_code";
    HookType["POST_WRITE_CODE"] = "post_write_code";
    HookType["PRE_RUN_COMMAND"] = "pre_run_command";
    HookType["POST_RUN_COMMAND"] = "post_run_command";
    HookType["PRE_MCP_TOOL_USE"] = "pre_mcp_tool_use";
    HookType["POST_MCP_TOOL_USE"] = "post_mcp_tool_use";
    HookType["PRE_USER_PROMPT"] = "pre_user_prompt";
    HookType["POST_CASCADE_RESPONSE"] = "post_cascade_response";
    HookType["POST_SETUP_WORKTREE"] = "post_setup_worktree";
})(HookType || (exports.HookType = HookType = {}));
// Error types
class SpyCodeError extends Error {
    constructor(message, code, details) {
        super(message);
        this.code = code;
        this.details = details;
        this.name = 'SpyCodeError';
    }
}
exports.SpyCodeError = SpyCodeError;
class CLIError extends SpyCodeError {
    constructor(message, details) {
        super(message, 'CLI_ERROR', details);
        this.name = 'CLIError';
    }
}
exports.CLIError = CLIError;
class MCPError extends SpyCodeError {
    constructor(message, details) {
        super(message, 'MCP_ERROR', details);
        this.name = 'MCPError';
    }
}
exports.MCPError = MCPError;
class CacheError extends SpyCodeError {
    constructor(message, details) {
        super(message, 'CACHE_ERROR', details);
        this.name = 'CacheError';
    }
}
exports.CacheError = CacheError;
// Notification types
var NotificationType;
(function (NotificationType) {
    NotificationType["INFO"] = "info";
    NotificationType["WARNING"] = "warning";
    NotificationType["ERROR"] = "error";
    NotificationType["SUCCESS"] = "success";
})(NotificationType || (exports.NotificationType = NotificationType = {}));
