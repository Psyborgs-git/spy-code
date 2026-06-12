"use strict";
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
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.McpServerManager = void 0;
const vscode = __importStar(require("vscode"));
const child_process_1 = require("child_process");
class McpServerManager {
    constructor() {
        this.mcpProcess = null;
        this.isRunning = false;
        this.onStatusChangeCallbacks = [];
    }
    onStatusChange(callback) {
        this.onStatusChangeCallbacks.push(callback);
    }
    notifyStatusChange() {
        for (const cb of this.onStatusChangeCallbacks) {
            cb(this.isRunning);
        }
    }
    async start() {
        if (this.isRunning)
            return;
        const config = vscode.workspace.getConfiguration('spy-code');
        const spyCodePath = config.get('path', 'spy-code');
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) {
            vscode.window.showErrorMessage('Spy-Code MCP requires an open workspace.');
            return;
        }
        const cwd = workspaceFolders[0].uri.fsPath;
        try {
            this.mcpProcess = (0, child_process_1.spawn)(spyCodePath, ['serve', '--mcp'], {
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
        }
        catch (e) {
            vscode.window.showErrorMessage(`Error starting MCP server: ${e}`);
        }
    }
    async stop() {
        if (!this.isRunning || !this.mcpProcess)
            return;
        this.mcpProcess.kill();
        this.isRunning = false;
        this.notifyStatusChange();
    }
}
exports.McpServerManager = McpServerManager;
//# sourceMappingURL=McpServerManager.js.map