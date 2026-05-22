/**
 * Core type definitions for spy-code integration
 */
export declare enum NodeKind {
    FUNCTION = "function",
    CLASS = "class",
    CONSTANT = "constant",
    DEPENDENCY = "dependency"
}
export declare enum Language {
    RUST = "rust",
    PYTHON = "python",
    TYPESCRIPT = "typescript",
    JAVASCRIPT = "javascript",
    GO = "go"
}
export declare enum EdgeKind {
    CALLS = "calls",
    IMPORTS = "imports",
    REFERENCES = "references"
}
export interface Node {
    id: string;
    kind: NodeKind;
    name: string;
    description?: string;
    signatures: Signature[];
    language: Language;
    filePath: string;
    startLine: number;
    endLine: number;
    gitSha?: string;
    renamedFrom?: string;
}
export interface Signature {
    params: Param[];
    returns?: string;
}
export interface Param {
    name: string;
    type?: string;
}
export interface Edge {
    from: Node;
    to: Node;
    kind: EdgeKind;
    confidence: number;
}
export interface SearchResult {
    node: Node;
    score: number;
}
export interface IndexStats {
    nodeCount: number;
    edgeCount: number;
    fileCount: number;
    lastIndexed?: string;
    lastGitSha?: string;
}
export interface GraphData {
    nodes: Node[];
    edges: Edge[];
}
export interface GraphFilter {
    filePath?: string;
    nodeKinds?: NodeKind[];
    languages?: Language[];
    edgeKinds?: EdgeKind[];
}
export interface SearchOptions {
    kind?: NodeKind;
    limit?: number;
    offset?: number;
}
export interface SemanticSearchOptions {
    limit?: number;
    threshold?: number;
}
export interface GitContext {
    branch: string;
    commit: string;
    changedFiles: string[];
    isDirty: boolean;
}
export interface IDEContext {
    ide: string;
    version: string;
    workspacePath: string;
    openFiles: string[];
    activeFile?: string;
    cursorPosition?: CursorPosition;
}
export interface CursorPosition {
    filePath: string;
    line: number;
    character: number;
}
export interface Selection {
    start: CursorPosition;
    end: CursorPosition;
}
export interface AgentState {
    agentId: string;
    sessionId: string;
    currentTask?: string;
    contextSize: number;
    lastActivity: Date;
}
export declare enum HookType {
    PRE_READ_CODE = "pre_read_code",
    POST_READ_CODE = "post_read_code",
    PRE_WRITE_CODE = "pre_write_code",
    POST_WRITE_CODE = "post_write_code",
    PRE_RUN_COMMAND = "pre_run_command",
    POST_RUN_COMMAND = "post_run_command",
    PRE_MCP_TOOL_USE = "pre_mcp_tool_use",
    POST_MCP_TOOL_USE = "post_mcp_tool_use",
    PRE_USER_PROMPT = "pre_user_prompt",
    POST_CASCADE_RESPONSE = "post_cascade_response",
    POST_SETUP_WORKTREE = "post_setup_worktree"
}
export interface HookContext {
    filePath: string;
    node?: Node;
    gitContext: GitContext;
    agentState: AgentState;
    ideContext: IDEContext;
    timestamp: Date;
}
export interface CodeModification {
    filePath: string;
    originalContent: string;
    modifiedContent: string;
    startLine: number;
    endLine: number;
}
export interface HookResult {
    continue: boolean;
    modifications?: CodeModification[];
    metadata?: Record<string, any>;
    error?: Error;
}
export type HookHandler = (context: HookContext) => Promise<HookResult>;
export interface IDEConfig {
    spyCodePath: string;
    dbPath: string;
    enableMCP: boolean;
    enableHooks: boolean;
    cacheEnabled: boolean;
    cacheTTL: number;
    maxCacheSize: number;
}
export interface IDEAdapter {
    initialize(): Promise<void>;
    activate(): Promise<void>;
    deactivate(): Promise<void>;
    showPanel(): void;
    hidePanel(): void;
    showNotification(message: string, type: NotificationType): void;
    getCurrentFile(): string | null;
    getCurrentSelection(): Selection | null;
    goToDefinition(filePath: string, line: number): Promise<void>;
    showReferences(references: Reference[]): Promise<void>;
    getConfig(): IDEConfig;
    updateConfig(config: Partial<IDEConfig>): void;
}
export interface CLIConfig {
    command: string;
    args: string[];
    timeout: number;
    env?: Record<string, string>;
}
export declare class SpyCodeError extends Error {
    code: string;
    details?: any | undefined;
    constructor(message: string, code: string, details?: any | undefined);
}
export declare class CLIError extends SpyCodeError {
    constructor(message: string, details?: any);
}
export declare class MCPError extends SpyCodeError {
    constructor(message: string, details?: any);
}
export declare class CacheError extends SpyCodeError {
    constructor(message: string, details?: any);
}
export declare enum NotificationType {
    INFO = "info",
    WARNING = "warning",
    ERROR = "error",
    SUCCESS = "success"
}
export interface Notification {
    message: string;
    type: NotificationType;
    timestamp: Date;
}
export interface Reference {
    filePath: string;
    line: number;
    character: number;
    node: Node;
}
