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
exports.SidebarProvider = void 0;
const vscode = __importStar(require("vscode"));
class SidebarProvider {
    constructor(_extensionUri, mcpServerManager, cliBridge) {
        this._extensionUri = _extensionUri;
        this.mcpServerManager = mcpServerManager;
        this.cliBridge = cliBridge;
        this.mcpServerManager.onStatusChange((isRunning) => {
            if (this._view) {
                this._view.webview.postMessage({ type: 'mcpStatus', isRunning });
            }
        });
    }
    resolveWebviewView(webviewView, context, _token) {
        this._view = webviewView;
        webviewView.webview.options = {
            enableScripts: true,
            localResourceRoots: [this._extensionUri]
        };
        webviewView.webview.html = this._getHtmlForWebview(webviewView.webview);
        webviewView.webview.onDidReceiveMessage(async (data) => {
            switch (data.type) {
                case 'startMcp':
                    await this.mcpServerManager.start();
                    break;
                case 'stopMcp':
                    await this.mcpServerManager.stop();
                    break;
                case 'getMcpStatus':
                    webviewView.webview.postMessage({ type: 'mcpStatus', isRunning: this.mcpServerManager.isRunning });
                    break;
                case 'search':
                    try {
                        const results = await this.cliBridge.search(data.query, { limit: 100 });
                        webviewView.webview.postMessage({ type: 'searchResults', results });
                    }
                    catch (e) {
                        webviewView.webview.postMessage({ type: 'error', message: e.message });
                    }
                    break;
                case 'semanticSearch':
                    try {
                        const results = await this.cliBridge.semanticSearch(data.query, { limit: 100 });
                        webviewView.webview.postMessage({ type: 'searchResults', results });
                    }
                    catch (e) {
                        webviewView.webview.postMessage({ type: 'error', message: e.message });
                    }
                    break;
                case 'getGraph':
                    try {
                        // Assuming a graphql query string or an endpoint.
                        // For now, we will execute spy-code query command.
                        // It might be better to just ask user to start HTTP server for advanced graph?
                        // Let's use simple CLI based graphql for graphData
                        const query = `
                    query {
                        graphData {
                            nodes { id kind name filePath }
                            edges { fromId toId kind }
                        }
                    }
                `;
                        // We will mock the execute command by requiring child_process to invoke spy-code directly
                        // since executeCommand is private on CLIBridge
                        const { spawn } = require('child_process');
                        const config = vscode.workspace.getConfiguration('spy-code');
                        const spyCodePath = config.get('path', 'spy-code');
                        // Use spawn to avoid Windows quoting issues with multi-line queries
                        const child = spawn(spyCodePath, ['query', query, '--json']);
                        let stdout = '';
                        let stderr = '';
                        child.stdout.on('data', (data) => { stdout += data.toString(); });
                        child.stderr.on('data', (data) => { stderr += data.toString(); });
                        child.on('close', (code) => {
                            if (code !== 0) {
                                webviewView.webview.postMessage({ type: 'error', message: stderr });
                                return;
                            }
                            try {
                                const graphData = JSON.parse(stdout);
                                webviewView.webview.postMessage({ type: 'graphData', graphData });
                            }
                            catch (err) {
                                webviewView.webview.postMessage({ type: 'error', message: err.message });
                            }
                        });
                    }
                    catch (e) {
                        webviewView.webview.postMessage({ type: 'error', message: e.message });
                    }
                    break;
                case 'goToDefinition':
                    const uri = vscode.Uri.file(data.filePath);
                    const document = await vscode.workspace.openTextDocument(uri);
                    const editor = await vscode.window.showTextDocument(document);
                    const position = new vscode.Position(data.line, 0);
                    editor.selection = new vscode.Selection(position, position);
                    editor.revealRange(new vscode.Range(position, position));
                    break;
                case 'updateConfig':
                    const config = vscode.workspace.getConfiguration('spy-code');
                    await config.update('path', data.config.path, vscode.ConfigurationTarget.Workspace);
                    await config.update('dbPath', data.config.dbPath, vscode.ConfigurationTarget.Workspace);
                    vscode.window.showInformationMessage('Configuration updated');
                    break;
                case 'getConfig':
                    const currentConfig = vscode.workspace.getConfiguration('spy-code');
                    webviewView.webview.postMessage({
                        type: 'configData',
                        config: {
                            path: currentConfig.get('path', 'spy-code'),
                            dbPath: currentConfig.get('dbPath', '.spy-code/graph.db')
                        }
                    });
                    break;
            }
        });
    }
    _getHtmlForWebview(webview) {
        const nonce = this.getNonce();
        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/cytoscape/3.26.0/cytoscape.min.js"></script>
    <style>
        body { font-family: var(--vscode-font-family); padding: 10px; color: var(--vscode-foreground); display: flex; flex-direction: column; height: 100vh; margin: 0; box-sizing: border-box;}
        h2 { font-size: 14px; margin-bottom: 5px; color: var(--vscode-textLink-foreground); }
        .section { margin-bottom: 20px; border: 1px solid var(--vscode-panel-border); padding: 10px; border-radius: 4px; }
        button { background: var(--vscode-button-background); color: var(--vscode-button-foreground); border: none; padding: 5px 10px; cursor: pointer; border-radius: 2px; }
        button:hover { background: var(--vscode-button-hoverBackground); }
        input { background: var(--vscode-input-background); color: var(--vscode-input-foreground); border: 1px solid var(--vscode-input-border); padding: 5px; width: 100%; box-sizing: border-box; margin-bottom: 5px;}
        .flex { display: flex; gap: 5px; }
        #cy { width: 100%; height: 300px; border: 1px solid var(--vscode-panel-border); margin-top: 10px; background: var(--vscode-editor-background); }
        .result-item { padding: 5px; border-bottom: 1px solid var(--vscode-panel-border); cursor: pointer; font-size: 12px; }
        .result-item:hover { background: var(--vscode-list-hoverBackground); }
        .status { font-weight: bold; }
        .status.running { color: #6bff6b; }
        .status.stopped { color: #ff6b6b; }
    </style>
</head>
<body>
    <div class="section">
        <h2>MCP Server Control</h2>
        <div class="flex">
            <button id="start-mcp">Start</button>
            <button id="stop-mcp">Stop</button>
        </div>
        <div style="margin-top: 5px;">Status: <span id="mcp-status" class="status stopped">Stopped</span></div>
    </div>

    <div class="section">
        <h2>Configuration</h2>
        <label>CLI Path:</label>
        <input type="text" id="config-path">
        <label>DB Path:</label>
        <input type="text" id="config-db">
        <button id="save-config">Save Config</button>
    </div>

    <div class="section" style="flex: 1; display: flex; flex-direction: column;">
        <h2>Search & Graph</h2>
        <div class="flex">
            <input type="text" id="search-input" placeholder="Search nodes...">
        </div>
        <div class="flex" style="margin-top: 5px;">
            <button id="search-btn">Search</button>
            <button id="semantic-search-btn">Semantic Search</button>
            <button id="load-graph-btn">Load Graph</button>
        </div>
        <div id="results" style="max-height: 150px; overflow-y: auto; margin-top: 10px;"></div>
        <div id="cy" style="flex: 1;"></div>
    </div>

    <script nonce="\${nonce}">
        const vscode = acquireVsCodeApi();
        let cy;

        // Init Cytoscape
        function initCy() {
            cy = cytoscape({
                container: document.getElementById('cy'),
                style: [
                    { selector: 'node', style: { 'label': 'data(name)', 'background-color': '#7eb8ff', 'color': '#fff', 'text-valign': 'center', 'font-size': '10px' } },
                    { selector: 'node[kind="CLASS"]', style: { 'background-color': '#ff6b6b' } },
                    { selector: 'node[kind="CONSTANT"]', style: { 'background-color': '#6bff6b' } },
                    { selector: 'edge', style: { 'width': 1, 'line-color': '#4a4a6a', 'target-arrow-color': '#4a4a6a', 'target-arrow-shape': 'triangle', 'curve-style': 'bezier' } }
                ],
                layout: { name: 'cose' }
            });

            cy.on('tap', 'node', function(evt){
                const node = evt.target;
                vscode.postMessage({ type: 'goToDefinition', filePath: node.data('filePath'), line: node.data('line') });
            });
        }

        initCy();

        document.getElementById('start-mcp').addEventListener('click', () => vscode.postMessage({ type: 'startMcp' }));
        document.getElementById('stop-mcp').addEventListener('click', () => vscode.postMessage({ type: 'stopMcp' }));

        document.getElementById('save-config').addEventListener('click', () => {
            vscode.postMessage({
                type: 'updateConfig',
                config: {
                    path: document.getElementById('config-path').value,
                    dbPath: document.getElementById('config-db').value
                }
            });
        });

        document.getElementById('search-btn').addEventListener('click', () => {
            const query = document.getElementById('search-input').value;
            document.getElementById('results').innerHTML = 'Searching...';
            vscode.postMessage({ type: 'search', query });
        });

        document.getElementById('semantic-search-btn').addEventListener('click', () => {
            const query = document.getElementById('search-input').value;
             document.getElementById('results').innerHTML = 'Searching...';
            vscode.postMessage({ type: 'semanticSearch', query });
        });

        document.getElementById('load-graph-btn').addEventListener('click', () => {
            vscode.postMessage({ type: 'getGraph' });
        });

        // Request initial status and config
        vscode.postMessage({ type: 'getMcpStatus' });
        vscode.postMessage({ type: 'getConfig' });

        window.addEventListener('message', event => {
            const message = event.data;
            switch (message.type) {
                case 'mcpStatus':
                    const el = document.getElementById('mcp-status');
                    el.textContent = message.isRunning ? 'Running' : 'Stopped';
                    el.className = 'status ' + (message.isRunning ? 'running' : 'stopped');
                    break;
                case 'configData':
                    document.getElementById('config-path').value = message.config.path;
                    document.getElementById('config-db').value = message.config.dbPath;
                    break;
                case 'searchResults':
                    const resultsContainer = document.getElementById('results');
                    resultsContainer.innerHTML = '';
                    if(!message.results || message.results.length === 0) {
                        resultsContainer.innerHTML = 'No results found.';
                        return;
                    }
                    message.results.forEach(r => {
                        const div = document.createElement('div');
                        div.className = 'result-item';

                        // Prevent XSS
                        const nameSpan = document.createElement('span');
                        nameSpan.textContent = r.node.name;

                        const kindSpan = document.createElement('span');
                        kindSpan.style.color = '#888';
                        kindSpan.textContent = \` (\${r.node.kind})\`;

                        const pathSpan = document.createElement('span');
                        pathSpan.textContent = \` - \${r.node.filePath}:\${r.node.startLine}\`;

                        div.appendChild(nameSpan);
                        div.appendChild(kindSpan);
                        div.appendChild(pathSpan);

                        div.addEventListener('click', () => {
                            vscode.postMessage({ type: 'goToDefinition', filePath: r.node.filePath, line: r.node.startLine });
                        });
                        resultsContainer.appendChild(div);
                    });
                    break;
                case 'graphData':
                    const data = message.graphData.graphData;
                    if(!data) return;
                    const elements = [];
                    data.nodes.forEach(n => elements.push({ data: { id: n.id, name: n.name, kind: n.kind, filePath: n.filePath, line: n.startLine } }));
                    data.edges.forEach(e => elements.push({ data: { source: e.fromId, target: e.toId, kind: e.kind } }));
                    cy.elements().remove();
                    cy.add(elements);
                    cy.layout({ name: 'cose' }).run();
                    break;
                case 'error':
                    document.getElementById('results').innerHTML = '<span style="color:red">Error: ' + message.message + '</span>';
                    break;
            }
        });
    </script>
</body>
</html>`;
    }
    getNonce() {
        let text = '';
        const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
        for (let i = 0; i < 32; i++) {
            text += possible.charAt(Math.floor(Math.random() * possible.length));
        }
        return text;
    }
}
exports.SidebarProvider = SidebarProvider;
//# sourceMappingURL=SidebarProvider.js.map