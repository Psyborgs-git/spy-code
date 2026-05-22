/**
 * VS Code Adapter - Adapter for VS Code Extension API
 * This adapter is reused for Cursor, Windsurf, and Antigravity (all VS Code-based)
 */

import * as vscode from 'vscode';
import {
  IDEAdapter,
  IDEConfig,
  NotificationType,
  CursorPosition,
  Selection,
  Reference
} from '@spy-code/integration-core';

export class VSCodeAdapter implements IDEAdapter {
  private context: vscode.ExtensionContext;
  private config: IDEConfig;
  private outputChannel: vscode.OutputChannel;

  constructor(context: vscode.ExtensionContext, config: Partial<IDEConfig> = {}) {
    this.context = context;
    this.config = {
      spyCodePath: config.spyCodePath || 'spy-code',
      dbPath: config.dbPath || '.spy-code/graph.db',
      enableMCP: config.enableMCP !== false,
      enableHooks: config.enableHooks !== false,
      cacheEnabled: config.cacheEnabled !== false,
      cacheTTL: config.cacheTTL || 300000,
      maxCacheSize: config.maxCacheSize || 1000
    };

    this.outputChannel = vscode.window.createOutputChannel('Spy-Code');
  }

  /**
   * Initialize the adapter
   */
  async initialize(): Promise<void> {
    this.log('Initializing VS Code adapter');

    // Register commands
    this.registerCommands();

    // Register file watchers
    this.registerFileWatchers();

    // Register status bar
    this.registerStatusBar();

    this.log('VS Code adapter initialized');
  }

  /**
   * Activate the adapter
   */
  async activate(): Promise<void> {
    this.log('Activating VS Code adapter');
    // Additional activation logic if needed
  }

  /**
   * Deactivate the adapter
   */
  async deactivate(): Promise<void> {
    this.log('Deactivating VS Code adapter');
    this.outputChannel.dispose();
  }

  /**
   * Show the spy-code panel
   */
  showPanel(): void {
    // Implementation will be in the extension
    vscode.commands.executeCommand('spy-code.showPanel');
  }

  /**
   * Hide the spy-code panel
   */
  hidePanel(): void {
    vscode.commands.executeCommand('workbench.action.closeActiveEditor');
  }

  /**
   * Show a notification
   */
  showNotification(message: string, type: NotificationType): void {
    switch (type) {
      case NotificationType.INFO:
        vscode.window.showInformationMessage(message);
        break;
      case NotificationType.WARNING:
        vscode.window.showWarningMessage(message);
        break;
      case NotificationType.ERROR:
        vscode.window.showErrorMessage(message);
        break;
      case NotificationType.SUCCESS:
        vscode.window.showInformationMessage(message);
        break;
    }
  }

  /**
   * Get the current file
   */
  getCurrentFile(): string | null {
    const activeEditor = vscode.window.activeTextEditor;
    if (!activeEditor) {
      return null;
    }
    return activeEditor.document.uri.fsPath;
  }

  /**
   * Get the current selection
   */
  getCurrentSelection(): Selection | null {
    const activeEditor = vscode.window.activeTextEditor;
    if (!activeEditor) {
      return null;
    }

    const selection = activeEditor.selection;
    return {
      start: {
        filePath: activeEditor.document.uri.fsPath,
        line: selection.start.line,
        character: selection.start.character
      },
      end: {
        filePath: activeEditor.document.uri.fsPath,
        line: selection.end.line,
        character: selection.end.character
      }
    };
  }

  /**
   * Go to definition at a specific location
   */
  async goToDefinition(filePath: string, line: number): Promise<void> {
    const uri = vscode.Uri.file(filePath);
    const document = await vscode.workspace.openTextDocument(uri);
    const editor = await vscode.window.showTextDocument(document);

    const position = new vscode.Position(line, 0);
    editor.selection = new vscode.Selection(position, position);
    editor.revealRange(new vscode.Range(position, position));
  }

  /**
   * Show references in a quick pick
   */
  async showReferences(references: Reference[]): Promise<void> {
    if (references.length === 0) {
      this.showNotification('No references found', NotificationType.INFO);
      return;
    }

    const items = references.map(ref => ({
      label: `${ref.node.name} (${ref.node.kind})`,
      description: `${ref.filePath}:${ref.line}`,
      reference: ref
    }));

    const selected = await vscode.window.showQuickPick(items, {
      placeHolder: 'Select a reference to navigate to'
    });

    if (selected) {
      await this.goToDefinition(selected.reference.filePath, selected.reference.line);
    }
  }

  /**
   * Get the current configuration
   */
  getConfig(): IDEConfig {
    return { ...this.config };
  }

  /**
   * Update the configuration
   */
  updateConfig(config: Partial<IDEConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Register VS Code commands
   */
  private registerCommands(): void {
    // Commands will be registered by the extension
    // This is a placeholder for the adapter to know what commands exist
  }

  /**
   * Register file watchers
   */
  private registerFileWatchers(): void {
    const watcher = vscode.workspace.createFileSystemWatcher(
      '**/*.{rs,py,ts,js,go}',
      false,
      false,
      false
    );

    watcher.onDidChange(uri => {
      this.log(`File changed: ${uri.fsPath}`);
      // Emit event through event bus
    });

    watcher.onDidCreate(uri => {
      this.log(`File created: ${uri.fsPath}`);
    });

    watcher.onDidDelete(uri => {
      this.log(`File deleted: ${uri.fsPath}`);
    });

    this.context.subscriptions.push(watcher);
  }

  /**
   * Register status bar item
   */
  private registerStatusBar(): void {
    const statusBarItem = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Right,
      100
    );

    statusBarItem.text = '$(search) Spy-Code';
    statusBarItem.command = 'spy-code.showPanel';
    statusBarItem.show();

    this.context.subscriptions.push(statusBarItem);
  }

  /**
   * Log message to output channel
   */
  private log(message: string): void {
    const timestamp = new Date().toISOString();
    this.outputChannel.appendLine(`[${timestamp}] ${message}`);
  }

  /**
   * Get workspace path
   */
  getWorkspacePath(): string | null {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders || workspaceFolders.length === 0) {
      return null;
    }
    return workspaceFolders[0].uri.fsPath;
  }

  /**
   * Get all open files
   */
  getOpenFiles(): string[] {
    return vscode.workspace.textDocuments
      .map(doc => doc.uri.fsPath)
      .filter(path => path);
  }

  /**
   * Get active file
   */
  getActiveFile(): string | null {
    return this.getCurrentFile();
  }

  /**
   * Get cursor position
   */
  getCursorPosition(): CursorPosition | null {
    const activeEditor = vscode.window.activeTextEditor;
    if (!activeEditor) {
      return null;
    }

    const position = activeEditor.selection.active;
    return {
      filePath: activeEditor.document.uri.fsPath,
      line: position.line,
      character: position.character
    };
  }

  /**
   * Read file content
   */
  async readFile(filePath: string): Promise<string> {
    const uri = vscode.Uri.file(filePath);
    const document = await vscode.workspace.openTextDocument(uri);
    return document.getText();
  }

  /**
   * Write file content
   */
  async writeFile(filePath: string, content: string): Promise<void> {
    const uri = vscode.Uri.file(filePath);
    const edit = new vscode.WorkspaceEdit();
    edit.replace(uri, new vscode.Range(0, 0, Number.MAX_VALUE, Number.MAX_VALUE), content);
    await vscode.workspace.applyEdit(edit);
  }

  /**
   * Get language for a file
   */
  getLanguage(filePath: string): string | null {
    const uri = vscode.Uri.file(filePath);
    const doc = vscode.workspace.textDocuments.find(d => d.uri.toString() === uri.toString());
    if (doc) {
      return doc.languageId;
    }
    // Try to get language from file extension
    const ext = filePath.split('.').pop();
    if (ext) {
      const langMap: Record<string, string> = {
        'ts': 'typescript',
        'js': 'javascript',
        'py': 'python',
        'rs': 'rust',
        'go': 'go'
      };
      return langMap[ext] || null;
    }
    return null;
  }
}
