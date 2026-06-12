import * as vscode from 'vscode';
import { spawn, ChildProcess } from 'child_process';

export class McpServerManager {
    private mcpProcess: ChildProcess | null = null;
    public isRunning: boolean = false;
    private onStatusChangeCallbacks: ((status: boolean) => void)[] = [];

    constructor() {}

    public onStatusChange(callback: (status: boolean) => void) {
        this.onStatusChangeCallbacks.push(callback);
    }

    private notifyStatusChange() {
        for (const cb of this.onStatusChangeCallbacks) {
            cb(this.isRunning);
        }
    }

    public async start() {
        if (this.isRunning) return;

        const config = vscode.workspace.getConfiguration('spy-code');
        const spyCodePath = config.get<string>('path', 'spy-code');

        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) {
            vscode.window.showErrorMessage('Spy-Code MCP requires an open workspace.');
            return;
        }

        const cwd = workspaceFolders[0].uri.fsPath;

        try {
            this.mcpProcess = spawn(spyCodePath, ['serve', '--mcp'], {
                cwd: cwd,
                stdio: 'ignore'
            });

            this.mcpProcess.on('error', (err) => {
                console.error('Failed to start MCP server', err);
                vscode.window.showErrorMessage(`Failed to start MCP server: ${err.message}`);
                this.isRunning = false;
                this.notifyStatusChange();
            });

            this.mcpProcess.on('exit', (code) => {
                console.log(`MCP server exited with code ${code}`);
                this.isRunning = false;
                this.notifyStatusChange();
            });

            this.isRunning = true;
            this.notifyStatusChange();

        } catch (e) {
            vscode.window.showErrorMessage(`Error starting MCP server: ${e}`);
        }
    }

    public async stop() {
        if (!this.isRunning || !this.mcpProcess) return;

        this.mcpProcess.kill();
        this.isRunning = false;
        this.notifyStatusChange();
    }
}
