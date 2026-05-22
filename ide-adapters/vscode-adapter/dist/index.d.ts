/**
 * VS Code Adapter - Adapter for VS Code Extension API
 * This adapter is reused for Cursor, Windsurf, and Antigravity (all VS Code-based)
 */
import * as vscode from 'vscode';
import { IDEAdapter, IDEConfig, NotificationType, CursorPosition, Selection, Reference } from '@spy-code/integration-core';
export declare class VSCodeAdapter implements IDEAdapter {
    private context;
    private config;
    private outputChannel;
    constructor(context: vscode.ExtensionContext, config?: Partial<IDEConfig>);
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
     * Show the spy-code panel
     */
    showPanel(): void;
    /**
     * Hide the spy-code panel
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
     * Show references in a quick pick
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
     * Register VS Code commands
     */
    private registerCommands;
    /**
     * Register file watchers
     */
    private registerFileWatchers;
    /**
     * Register status bar item
     */
    private registerStatusBar;
    /**
     * Log message to output channel
     */
    private log;
    /**
     * Get workspace path
     */
    getWorkspacePath(): string | null;
    /**
     * Get all open files
     */
    getOpenFiles(): string[];
    /**
     * Get active file
     */
    getActiveFile(): string | null;
    /**
     * Get cursor position
     */
    getCursorPosition(): CursorPosition | null;
    /**
     * Read file content
     */
    readFile(filePath: string): Promise<string>;
    /**
     * Write file content
     */
    writeFile(filePath: string, content: string): Promise<void>;
    /**
     * Get language for a file
     */
    getLanguage(filePath: string): string | null;
}
