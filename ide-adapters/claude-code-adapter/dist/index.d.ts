/**
 * Claude Code Adapter - Adapter for Claude Code extension API
 * Claude Code has built-in MCP support and workflow system
 */
import { IDEAdapter, IDEConfig, NotificationType, Selection, Reference } from '@spy-code/integration-core';
export declare class ClaudeCodeAdapter implements IDEAdapter {
    private config;
    private claudeCodePath;
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
     * Show the spy-code panel (not applicable in Claude Code)
     */
    showPanel(): void;
    /**
     * Hide the spy-code panel (not applicable in Claude Code)
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
     * Setup MCP configuration for Claude Code
     */
    private setupMCPConfig;
    /**
     * Register Claude Code skills
     */
    private registerSkills;
    /**
     * Register Claude Code workflows
     */
    private registerWorkflows;
    /**
     * Get workspace path
     */
    getWorkspacePath(): string;
    /**
     * Set workspace path
     */
    setWorkspacePath(path: string): void;
    /**
     * Execute a Claude Code command
     */
    executeCommand(command: string, args: string[]): Promise<string>;
    /**
     * Get Claude Code version
     */
    getVersion(): Promise<string>;
}
