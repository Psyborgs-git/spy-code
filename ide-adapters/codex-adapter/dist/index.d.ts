/**
 * OpenAI Codex Adapter - Adapter for OpenAI Codex SDK
 * Codex has both TypeScript and Python SDKs, and supports agent workflows
 */
import { IDEAdapter, IDEConfig, NotificationType, Selection, Reference } from '@spy-code/integration-core';
export declare class CodexAdapter implements IDEAdapter {
    private config;
    private codexAgentId;
    private workspacePath;
    constructor(config?: Partial<IDEConfig>);
    /**
     * Initialize the adapter
     */
    initialize(): Promise<void>;
    /**
     * Activate the adapter
     */
    activate(): Promise<void>;
    /**
     * Deactivate the adapter
     */
    deactivate(): Promise<void>;
    /**
     * Show the spy-code panel (not applicable in Codex)
     */
    showPanel(): void;
    /**
     * Hide the spy-code panel (not applicable in Codex)
     */
    hidePanel(): void;
    /**
     * Show a notification
     */
    showNotification(message: string, type: NotificationType): void;
    /**
     * Get the current file
     */
    getCurrentFile(): string | null;
    /**
     * Get the current selection
     */
    getCurrentSelection(): Selection | null;
    /**
     * Go to definition at a specific location
     */
    goToDefinition(filePath: string, line: number): Promise<void>;
    /**
     * Show references
     */
    showReferences(references: Reference[]): Promise<void>;
    /**
     * Get the current configuration
     */
    getConfig(): IDEConfig;
    /**
     * Update the configuration
     */
    updateConfig(config: Partial<IDEConfig>): void;
    /**
     * Setup Codex agent configuration
     */
    private setupCodexConfig;
    /**
     * Register Codex agent templates
     */
    private registerAgentTemplates;
    /**
     * Get workspace path
     */
    getWorkspacePath(): string;
    /**
     * Set workspace path
     */
    setWorkspacePath(path: string): void;
    /**
     * Get Codex agent ID
     */
    getAgentId(): string;
    /**
     * Execute Codex agent
     */
    executeAgent(agentId: string, task: string): Promise<any>;
}
