/**
 * GitHub Copilot Adapter - Adapter for GitHub Copilot Extensions API
 * Copilot supports MCP protocol and has an Extensions SDK
 */
import { IDEAdapter, IDEConfig, NotificationType, Selection, Reference } from '@spy-code/integration-core';
export declare class CopilotAdapter implements IDEAdapter {
    private config;
    private copilotExtensionId;
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
     * Show the spy-code panel (not applicable in Copilot)
     */
    showPanel(): void;
    /**
     * Hide the spy-code panel (not applicable in Copilot)
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
     * Setup Copilot extension configuration
     */
    private setupCopilotConfig;
    /**
     * Register Copilot skill/tool
     */
    private registerCopilotSkill;
    /**
     * Get workspace path
     */
    getWorkspacePath(): string;
    /**
     * Set workspace path
     */
    setWorkspacePath(path: string): void;
    /**
     * Get Copilot extension ID
     */
    getExtensionId(): string;
}
