/**
 * Sidebar Provider - Webview panel for spy-code UI
 */

import * as vscode from 'vscode';
import { CLIBridge, MCPClient, CacheManager } from '@spy-code/integration-core';

export class SidebarProvider implements vscode.WebviewViewProvider {
  private _view?: vscode.WebviewView;
  private cliBridge: CLIBridge;
  private mcpClient?: MCPClient;
  private cacheManager?: CacheManager;

  constructor(
    private readonly _extensionUri: vscode.Uri,
    cliBridge: CLIBridge,
    mcpClient?: MCPClient,
    cacheManager?: CacheManager
  ) {
    this.cliBridge = cliBridge;
    this.mcpClient = mcpClient;
    this.cacheManager = cacheManager;
  }

  public resolveWebviewView(
    webviewView: vscode.WebviewView
  ): void {
    this._view = webviewView;

    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [this._extensionUri]
    };

    webviewView.webview.html = this._getHtmlForWebview(webviewView.webview);

    webviewView.webview.onDidReceiveMessage(async (data) => {
      switch (data.type) {
        case 'search':
          await this.handleSearch(data.query, data.options);
          break;
        case 'semanticSearch':
          await this.handleSemanticSearch(data.query, data.options);
          break;
        case 'getNode':
          await this.handleGetNode(data.nodeId);
          break;
        case 'getCallers':
          await this.handleGetCallers(data.nodeId, data.depth);
          break;
        case 'getCallees':
          await this.handleGetCallees(data.nodeId, data.depth);
          break;
        case 'reindex':
          await this.handleReindex(data.full);
          break;
        case 'getStats':
          await this.handleGetStats();
          break;
        case 'goToDefinition':
          await this.handleGoToDefinition(data.filePath, data.line);
          break;
      }
    });
  }

  private _getHtmlForWebview(webview: vscode.Webview): string {
    const nonce = this.getNonce();

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; script-src 'nonce-${nonce}';">
  <title>Spy-Code</title>
  <style>
    body {
      font-family: var(--vscode-font-family);
      font-size: var(--vscode-font-size);
      color: var(--vscode-foreground);
      background-color: var(--vscode-editor-background);
      padding: 10px;
      margin: 0;
    }
    .container {
      display: flex;
      flex-direction: column;
      gap: 10px;
    }
    .search-box {
      display: flex;
      gap: 5px;
    }
    input {
      flex: 1;
      padding: 5px;
      background-color: var(--vscode-input-background);
      color: var(--vscode-input-foreground);
      border: 1px solid var(--vscode-input-border);
      border-radius: 2px;
    }
    button {
      padding: 5px 10px;
      background-color: var(--vscode-button-background);
      color: var(--vscode-button-foreground);
      border: none;
      border-radius: 2px;
      cursor: pointer;
    }
    button:hover {
      background-color: var(--vscode-button-hoverBackground);
    }
    .results {
      flex: 1;
      overflow-y: auto;
      border: 1px solid var(--vscode-panel-border);
      border-radius: 2px;
      padding: 5px;
    }
    .result-item {
      padding: 5px;
      border-bottom: 1px solid var(--vscode-panel-border);
      cursor: pointer;
    }
    .result-item:hover {
      background-color: var(--vscode-list-hoverBackground);
    }
    .result-name {
      font-weight: bold;
    }
    .result-kind {
      color: var(--vscode-descriptionForeground);
      font-size: 0.9em;
    }
    .result-location {
      color: var(--vscode-textLink-foreground);
      font-size: 0.85em;
    }
    .stats {
      padding: 10px;
      background-color: var(--vscode-panel-background);
      border: 1px solid var(--vscode-panel-border);
      border-radius: 2px;
    }
    .loading {
      text-align: center;
      padding: 20px;
      color: var(--vscode-descriptionForeground);
    }
  </style>
</head>
<body>
  <div class="container">
    <div class="search-box">
      <input type="text" id="searchInput" placeholder="Search codebase..." />
      <button id="searchBtn">Search</button>
      <button id="semanticBtn">Semantic</button>
    </div>
    <div class="results" id="results">
      <div class="loading">Enter a search query to begin</div>
    </div>
    <div class="stats" id="stats">
      <button id="statsBtn">Show Stats</button>
      <button id="reindexBtn">Reindex</button>
    </div>
  </div>
  <script nonce="${nonce}">
    const vscode = acquireVsCodeApi();
    
    document.getElementById('searchBtn').addEventListener('click', () => {
      const query = document.getElementById('searchInput').value;
      vscode.postMessage({ type: 'search', query, options: {} });
    });
    
    document.getElementById('semanticBtn').addEventListener('click', () => {
      const query = document.getElementById('searchInput').value;
      vscode.postMessage({ type: 'semanticSearch', query, options: {} });
    });
    
    document.getElementById('statsBtn').addEventListener('click', () => {
      vscode.postMessage({ type: 'getStats' });
    });
    
    document.getElementById('reindexBtn').addEventListener('click', () => {
      vscode.postMessage({ type: 'reindex', full: false });
    });
    
    window.addEventListener('message', event => {
      const message = event.data;
      const resultsDiv = document.getElementById('results');
      
      if (message.type === 'searchResults') {
        if (message.results.length === 0) {
          resultsDiv.innerHTML = '<div class="loading">No results found</div>';
        } else {
          resultsDiv.innerHTML = message.results.map(r => \`
            <div class="result-item" data-filepath="\${r.node.filePath}" data-line="\${r.node.startLine}">
              <div class="result-name">\${r.node.name}</div>
              <div class="result-kind">\${r.node.kind}</div>
              <div class="result-location">\${r.node.filePath}:\${r.node.startLine}</div>
            </div>
          \`).join('');
          
          // Add click handlers
          document.querySelectorAll('.result-item').forEach(item => {
            item.addEventListener('click', () => {
              const filePath = item.dataset.filepath;
              const line = parseInt(item.dataset.line);
              vscode.postMessage({ type: 'goToDefinition', filePath, line });
            });
          });
        }
      }
      
      if (message.type === 'stats') {
        const statsDiv = document.getElementById('stats');
        statsDiv.innerHTML = \`
          <div>Nodes: \${message.stats.nodeCount}</div>
          <div>Edges: \${message.stats.edgeCount}</div>
          <div>Files: \${message.stats.fileCount}</div>
          <button id="statsBtn">Refresh Stats</button>
          <button id="reindexBtn">Reindex</button>
        \`;
        
        document.getElementById('statsBtn').addEventListener('click', () => {
          vscode.postMessage({ type: 'getStats' });
        });
        
        document.getElementById('reindexBtn').addEventListener('click', () => {
          vscode.postMessage({ type: 'reindex', full: false });
        });
      }
      
      if (message.type === 'loading') {
        resultsDiv.innerHTML = '<div class="loading">Searching...</div>';
      }
    });
  </script>
</body>
</html>`;
  }

  private async handleSearch(query: string, options: any) {
    this._view?.webview.postMessage({ type: 'loading' });

    try {
      const results = await this.cliBridge.search(query, options);
      this._view?.webview.postMessage({ type: 'searchResults', results });
    } catch (error) {
      this._view?.webview.postMessage({ type: 'error', message: String(error) });
    }
  }

  private async handleSemanticSearch(query: string, options: any) {
    this._view?.webview.postMessage({ type: 'loading' });

    try {
      const results = await this.cliBridge.semanticSearch(query, options);
      this._view?.webview.postMessage({ type: 'searchResults', results });
    } catch (error) {
      this._view?.webview.postMessage({ type: 'error', message: String(error) });
    }
  }

  private async handleGetNode(nodeId: string) {
    try {
      const node = await this.cliBridge.getNode(nodeId);
      this._view?.webview.postMessage({ type: 'nodeResult', node });
    } catch (error) {
      this._view?.webview.postMessage({ type: 'error', message: String(error) });
    }
  }

  private async handleGetCallers(nodeId: string, depth: number) {
    try {
      const edges = await this.cliBridge.getCallers(nodeId, depth);
      this._view?.webview.postMessage({ type: 'callersResult', edges });
    } catch (error) {
      this._view?.webview.postMessage({ type: 'error', message: String(error) });
    }
  }

  private async handleGetCallees(nodeId: string, depth: number) {
    try {
      const edges = await this.cliBridge.getCallees(nodeId, depth);
      this._view?.webview.postMessage({ type: 'calleesResult', edges });
    } catch (error) {
      this._view?.webview.postMessage({ type: 'error', message: String(error) });
    }
  }

  private async handleReindex(full: boolean) {
    try {
      const stats = await this.cliBridge.reindex(full);
      this._view?.webview.postMessage({ type: 'stats', stats });
    } catch (error) {
      this._view?.webview.postMessage({ type: 'error', message: String(error) });
    }
  }

  private async handleGetStats() {
    try {
      const stats = await this.cliBridge.getStats();
      this._view?.webview.postMessage({ type: 'stats', stats });
    } catch (error) {
      this._view?.webview.postMessage({ type: 'error', message: String(error) });
    }
  }

  private async handleGoToDefinition(filePath: string, line: number) {
    const uri = vscode.Uri.file(filePath);
    const document = await vscode.workspace.openTextDocument(uri);
    const editor = await vscode.window.showTextDocument(document);
    const position = new vscode.Position(line, 0);
    editor.selection = new vscode.Selection(position, position);
    editor.revealRange(new vscode.Range(position, position));
  }

  private getNonce(): string {
    let text = '';
    const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    for (let i = 0; i < 32; i++) {
      text += possible.charAt(Math.floor(Math.random() * possible.length));
    }
    return text;
  }
}
