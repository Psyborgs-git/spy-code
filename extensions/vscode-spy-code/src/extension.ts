import * as vscode from 'vscode';
import { MCPClient } from '@spy-code/integration-core';

let mcpClient: MCPClient | null = null;
let graphViewProvider: GraphViewProvider | null = null;

export function activate(context: vscode.ExtensionContext) {
  console.log('Spy-Code extension is now active');

  // Initialize MCP client
  initializeMCPClient(context);

  // Register commands
  const commands = [
    vscode.commands.registerCommand('spy-code.search', handleSearch),
    vscode.commands.registerCommand('spy-code.findCallers', handleFindCallers),
    vscode.commands.registerCommand('spy-code.findCallees', handleFindCallees),
    vscode.commands.registerCommand('spy-code.showNode', handleShowNode),
    vscode.commands.registerCommand('spy-code.index', handleIndex),
    vscode.commands.registerCommand('spy-code.openGraph', handleOpenGraph),
    vscode.commands.registerCommand('spy-code.stats', handleStats)
  ];

  // Register graph view provider
  graphViewProvider = new GraphViewProvider(context.extensionUri);
  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider('spy-code.graphView', graphViewProvider)
  );

  context.subscriptions.push(...commands);
}

async function initializeMCPClient(context: vscode.ExtensionContext) {
  try {
    mcpClient = new MCPClient();
    await mcpClient.connect();
    await mcpClient.initialize();
    console.log('Spy-Code MCP client connected');
  } catch (error) {
    console.error('Failed to connect to Spy-Code MCP server:', error);
    vscode.window.showErrorMessage(
      'Failed to connect to Spy-Code. Make sure spy-code is installed and the MCP server is running.'
    );
  }
}

async function handleSearch() {
  if (!mcpClient) {
    vscode.window.showWarningMessage('Spy-Code MCP client not connected');
    return;
  }

  const query = await vscode.window.showInputBox({
    prompt: 'Enter search query',
    placeHolder: 'e.g., authenticate, user, process'
  });

  if (!query) {
    return;
  }

  try {
    const results = await mcpClient.search(query, { limit: 20 });
    
    if (results.length === 0) {
      vscode.window.showInformationMessage('No results found');
      return;
    }

    // Show quick pick with results
    const items = results.map(r => ({
      label: r.node.name,
      description: `${r.node.kind} - ${r.node.filePath}`,
      detail: r.node.description || '',
      node: r.node
    }));

    const selected = await vscode.window.showQuickPick(items, {
      placeHolder: 'Select a result'
    });

    if (selected) {
      await showNodeDetails(selected.node);
    }
  } catch (error) {
    vscode.window.showErrorMessage(`Search failed: ${error}`);
  }
}

async function handleFindCallers() {
  if (!mcpClient) {
    vscode.window.showWarningMessage('Spy-Code MCP client not connected');
    return;
  }

  const nodeId = await getNodeIdFromCursor();
  if (!nodeId) {
    return;
  }

  try {
    const callers = await mcpClient.getCallers(nodeId, 2);
    
    if (callers.length === 0) {
      vscode.window.showInformationMessage('No callers found');
      return;
    }

    // Show callers in a new document
    const document = await vscode.workspace.openTextDocument({
      content: formatCallers(callers),
      language: 'markdown'
    });
    await vscode.window.showTextDocument(document);
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to find callers: ${error}`);
  }
}

async function handleFindCallees() {
  if (!mcpClient) {
    vscode.window.showWarningMessage('Spy-Code MCP client not connected');
    return;
  }

  const nodeId = await getNodeIdFromCursor();
  if (!nodeId) {
    return;
  }

  try {
    const callees = await mcpClient.getCallees(nodeId, 2);
    
    if (callees.length === 0) {
      vscode.window.showInformationMessage('No callees found');
      return;
    }

    // Show callees in a new document
    const document = await vscode.workspace.openTextDocument({
      content: formatCallees(callees),
      language: 'markdown'
    });
    await vscode.window.showTextDocument(document);
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to find callees: ${error}`);
  }
}

async function handleShowNode() {
  if (!mcpClient) {
    vscode.window.showWarningMessage('Spy-Code MCP client not connected');
    return;
  }

  const nodeId = await getNodeIdFromCursor();
  if (!nodeId) {
    return;
  }

  try {
    const node = await mcpClient.getNode(nodeId);
    if (!node) {
      vscode.window.showInformationMessage('Node not found');
      return;
    }

    await showNodeDetails(node);
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to get node: ${error}`);
  }
}

async function handleIndex() {
  if (!mcpClient) {
    vscode.window.showWarningMessage('Spy-Code MCP client not connected');
    return;
  }

  const progress = await vscode.window.withProgress({
    location: vscode.ProgressLocation.Notification,
    title: 'Indexing codebase...',
    cancellable: false
  }, async () => {
    try {
      await mcpClient.callTool('index', {});
      vscode.window.showInformationMessage('Codebase indexed successfully');
    } catch (error) {
      vscode.window.showErrorMessage(`Indexing failed: ${error}`);
    }
  });
}

async function handleOpenGraph() {
  if (!graphViewProvider) {
    vscode.window.showWarningMessage('Graph view not available');
    return;
  }

  // Focus the graph view
  await vscode.commands.executeCommand('spy-code.graphView.focus');
  
  // Load graph data
  if (mcpClient) {
    try {
      const graphData = await mcpClient.queryGraph(`
        {
          graphData {
            nodes {
              id
              name
              kind
              filePath
            }
            edges {
              from { id }
              to { id }
              kind
            }
          }
        }
      `);
      graphViewProvider.updateGraph(graphData);
    } catch (error) {
      vscode.window.showErrorMessage(`Failed to load graph: ${error}`);
    }
  }
}

async function handleStats() {
  if (!mcpClient) {
    vscode.window.showWarningMessage('Spy-Code MCP client not connected');
    return;
  }

  try {
    const stats = await mcpClient.getStats();
    
    const message = `
Spy-Code Statistics:
- Nodes: ${stats.nodeCount}
- Edges: ${stats.edgeCount}
- Files: ${stats.fileCount}
- Last Indexed: ${stats.lastIndexed || 'Never'}
- Last Git SHA: ${stats.lastGitSha || 'N/A'}
    `.trim();

    vscode.window.showInformationMessage(message);
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to get stats: ${error}`);
  }
}

async function getNodeIdFromCursor(): Promise<string | undefined> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showWarningMessage('No active editor');
    return;
  }

  const document = editor.document;
  const filePath = document.uri.fsPath;
  const workspacePath = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  
  if (!workspacePath) {
    vscode.window.showWarningMessage('No workspace folder');
    return;
  }

  // Convert file path to node ID format
  const relativePath = filePath.replace(workspacePath + '/', '').replace(/\.[^/.]+$/, '');
  const nodeId = `${relativePath.replace(/\//g, ':')}:_:_`;
  
  // Ask user to specify the symbol
  const symbol = await vscode.window.showInputBox({
    prompt: 'Enter symbol name',
    placeHolder: 'e.g., function_name, ClassName'
  });

  if (!symbol) {
    return;
  }

  return `${relativePath.replace(/\//g, ':')}:_:${symbol}`;
}

async function showNodeDetails(node: any) {
  const content = `
# ${node.name}

**Kind:** ${node.kind}
**Language:** ${node.language}
**File:** ${node.filePath}
**Location:** Line ${node.startLine} - ${node.endLine}

## Description
${node.description || 'No description'}

## Signatures
${node.signatures.map((sig: any) => `
- Parameters: ${sig.params.map((p: any) => `${p.name}: ${p.type || 'any'}`).join(', ')}
- Returns: ${sig.returns || 'void'}
`).join('\n')}
  `.trim();

  const document = await vscode.workspace.openTextDocument({
    content,
    language: 'markdown'
  });
  await vscode.window.showTextDocument(document);

  // Open the file at the location
  const uri = vscode.Uri.file(node.filePath);
  const doc = await vscode.workspace.openTextDocument(uri);
  await vscode.window.showTextDocument(doc, {
    selection: new vscode.Range(
      new vscode.Position(node.startLine - 1, 0),
      new vscode.Position(node.startLine - 1, 0)
    )
  });
}

function formatCallers(callers: any[]): string {
  return `# Callers

${callers.map((edge, i) => `
${i + 1}. **${edge.from.name}**
   - File: ${edge.from.filePath}
   - Kind: ${edge.kind}
   - Confidence: ${edge.confidence}
`).join('\n')}
  `.trim();
}

function formatCallees(callees: any[]): string {
  return `# Callees

${callees.map((edge, i) => `
${i + 1}. **${edge.to.name}**
   - File: ${edge.to.filePath}
   - Kind: ${edge.kind}
   - Confidence: ${edge.confidence}
`).join('\n')}
  `.trim();
}

export function deactivate() {
  if (mcpClient) {
    mcpClient.disconnect();
  }
}

class GraphViewProvider implements vscode.WebviewViewProvider {
  private _view?: vscode.WebviewView;
  private _extensionUri: vscode.Uri;

  constructor(extensionUri: vscode.Uri) {
    this._extensionUri = extensionUri;
  }

  public resolveWebviewView(
    webviewView: vscode.WebviewView,
    context: vscode.WebviewViewResolveContext,
    _token: vscode.CancellationToken
  ) {
    this._view = webviewView;

    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [this._extensionUri]
    };

    webviewView.webview.html = this.getHtmlForWebview();
  }

  public updateGraph(graphData: any) {
    if (this._view) {
      this._view.webview.postMessage({
        type: 'updateGraph',
        data: graphData
      });
    }
  }

  private getHtmlForWebview(): string {
    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Spy-Code Graph</title>
  <style>
    body {
      font-family: var(--vscode-font-family);
      padding: 10px;
      color: var(--vscode-foreground);
    }
    #graph-container {
      width: 100%;
      height: 400px;
      border: 1px solid var(--vscode-panel-border);
      border-radius: 4px;
      overflow: auto;
    }
    .node {
      padding: 8px;
      margin: 4px;
      background: var(--vscode-editor-background);
      border: 1px solid var(--vscode-panel-border);
      border-radius: 4px;
      cursor: pointer;
    }
    .node:hover {
      background: var(--vscode-list-hoverBackground);
    }
    .node-name {
      font-weight: bold;
    }
    .node-kind {
      font-size: 0.8em;
      color: var(--vscode-descriptionForeground);
    }
  </style>
</head>
<body>
  <div id="graph-container">
    <p>Graph visualization will appear here</p>
  </div>
  <script>
    const vscode = acquireVsCodeApi();
    
    window.addEventListener('message', event => {
      const message = event.data;
      if (message.type === 'updateGraph') {
        renderGraph(message.data);
      }
    });
    
    function renderGraph(data) {
      const container = document.getElementById('graph-container');
      container.innerHTML = '';
      
      if (!data.graphData) {
        container.innerHTML = '<p>No graph data available</p>';
        return;
      }
      
      const nodes = data.graphData.nodes || [];
      const edges = data.graphData.edges || [];
      
      // Simple node list visualization
      nodes.forEach(node => {
        const div = document.createElement('div');
        div.className = 'node';
        div.innerHTML = \`
          <div class="node-name">\${node.name}</div>
          <div class="node-kind">\${node.kind} - \${node.filePath}</div>
        \`;
        div.onclick = () => {
          vscode.postMessage({
            type: 'selectNode',
            nodeId: node.id
          });
        };
        container.appendChild(div);
      });
    }
  </script>
</body>
</html>`;
  }
}
