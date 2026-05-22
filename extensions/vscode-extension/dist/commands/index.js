"use strict";
/**
 * Command Registration - Register all VS Code commands
 */
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
exports.registerCommands = registerCommands;
const vscode = __importStar(require("vscode"));
const integration_core_1 = require("@spy-code/integration-core");
function registerCommands(context, adapter, cliBridge, mcpClient, cacheManager, sidebarProvider) {
    // Show panel command
    const showPanelCommand = vscode.commands.registerCommand('spy-code.showPanel', () => {
        vscode.commands.executeCommand('spy-code.sidebar.focus');
    });
    context.subscriptions.push(showPanelCommand);
    // Search command
    const searchCommand = vscode.commands.registerCommand('spy-code.search', async () => {
        const query = await vscode.window.showInputBox({
            placeHolder: 'Enter search query',
            prompt: 'Search the codebase'
        });
        if (query) {
            try {
                integration_core_1.eventBus.emit(integration_core_1.EventType.SEARCH_STARTED, { query, options: {} });
                const results = await cliBridge.search(query);
                integration_core_1.eventBus.emit(integration_core_1.EventType.SEARCH_COMPLETED, { query, results });
                // Show results in quick pick
                const items = results.map(r => ({
                    label: r.node.name,
                    description: `${r.node.kind} - ${r.node.filePath}:${r.node.startLine}`,
                    result: r
                }));
                const selected = await vscode.window.showQuickPick(items, {
                    placeHolder: 'Select a result to navigate to'
                });
                if (selected) {
                    await adapter.goToDefinition(selected.result.node.filePath, selected.result.node.startLine);
                }
            }
            catch (error) {
                integration_core_1.eventBus.emit(integration_core_1.EventType.SEARCH_FAILED, { query, error });
                vscode.window.showErrorMessage(`Search failed: ${error}`);
            }
        }
    });
    context.subscriptions.push(searchCommand);
    // Semantic search command
    const semanticSearchCommand = vscode.commands.registerCommand('spy-code.semanticSearch', async () => {
        const query = await vscode.window.showInputBox({
            placeHolder: 'Enter natural language query',
            prompt: 'Semantic search the codebase'
        });
        if (query) {
            try {
                const results = await cliBridge.semanticSearch(query);
                const items = results.map(r => ({
                    label: r.node.name,
                    description: `${r.node.kind} - ${r.node.filePath}:${r.node.startLine} (score: ${r.score.toFixed(2)})`,
                    result: r
                }));
                const selected = await vscode.window.showQuickPick(items, {
                    placeHolder: 'Select a result to navigate to'
                });
                if (selected) {
                    await adapter.goToDefinition(selected.result.node.filePath, selected.result.node.startLine);
                }
            }
            catch (error) {
                vscode.window.showErrorMessage(`Semantic search failed: ${error}`);
            }
        }
    });
    context.subscriptions.push(semanticSearchCommand);
    // Go to definition command
    const goToDefinitionCommand = vscode.commands.registerCommand('spy-code.goToDefinition', async () => {
        const activeEditor = vscode.window.activeTextEditor;
        if (!activeEditor) {
            return;
        }
        const position = activeEditor.selection.active;
        const line = activeEditor.document.lineAt(position.line);
        const word = line.text.split(/\s+/).find(w => position.character >= line.text.indexOf(w) && position.character < line.text.indexOf(w) + w.length);
        if (word) {
            try {
                const results = await cliBridge.search(word, { limit: 10 });
                if (results.length > 0) {
                    await adapter.goToDefinition(results[0].node.filePath, results[0].node.startLine);
                }
                else {
                    vscode.window.showInformationMessage('No definition found');
                }
            }
            catch (error) {
                vscode.window.showErrorMessage(`Go to definition failed: ${error}`);
            }
        }
    });
    context.subscriptions.push(goToDefinitionCommand);
    // Find references command
    const findReferencesCommand = vscode.commands.registerCommand('spy-code.findReferences', async () => {
        const activeEditor = vscode.window.activeTextEditor;
        if (!activeEditor) {
            return;
        }
        const position = activeEditor.selection.active;
        const line = activeEditor.document.lineAt(position.line);
        const word = line.text.split(/\s+/).find(w => position.character >= line.text.indexOf(w) && position.character < line.text.indexOf(w) + w.length);
        if (word) {
            try {
                const results = await cliBridge.search(word, { limit: 50 });
                const references = results.map(r => ({
                    filePath: r.node.filePath,
                    line: r.node.startLine,
                    character: 0,
                    node: r.node
                }));
                await adapter.showReferences(references);
            }
            catch (error) {
                vscode.window.showErrorMessage(`Find references failed: ${error}`);
            }
        }
    });
    context.subscriptions.push(findReferencesCommand);
    // Show call graph command
    const showCallGraphCommand = vscode.commands.registerCommand('spy-code.showCallGraph', async () => {
        const activeEditor = vscode.window.activeTextEditor;
        if (!activeEditor) {
            return;
        }
        const position = activeEditor.selection.active;
        const line = activeEditor.document.lineAt(position.line);
        const word = line.text.split(/\s+/).find(w => position.character >= line.text.indexOf(w) && position.character < line.text.indexOf(w) + w.length);
        if (word) {
            try {
                const results = await cliBridge.search(word, { limit: 1 });
                if (results.length > 0) {
                    const node = results[0].node;
                    const callers = await cliBridge.getCallers(node.id, 2);
                    const callees = await cliBridge.getCallees(node.id, 2);
                    // Show call graph in a new document
                    const graphContent = generateCallGraphText(node, callers, callees);
                    const doc = await vscode.workspace.openTextDocument({
                        language: 'plaintext',
                        content: graphContent
                    });
                    await vscode.window.showTextDocument(doc);
                }
            }
            catch (error) {
                vscode.window.showErrorMessage(`Show call graph failed: ${error}`);
            }
        }
    });
    context.subscriptions.push(showCallGraphCommand);
    // Reindex command
    const reindexCommand = vscode.commands.registerCommand('spy-code.reindex', async () => {
        const full = await vscode.window.showQuickPick(['Incremental', 'Full'], { placeHolder: 'Select reindex type' });
        if (full) {
            try {
                integration_core_1.eventBus.emit(integration_core_1.EventType.INDEX_STARTED, { full: full === 'Full' });
                await vscode.window.withProgress({
                    location: vscode.ProgressLocation.Notification,
                    title: 'Reindexing codebase...',
                    cancellable: false
                }, async () => {
                    const stats = await cliBridge.reindex(full === 'Full');
                    integration_core_1.eventBus.emit(integration_core_1.EventType.INDEX_COMPLETED, { stats });
                    vscode.window.showInformationMessage(`Reindex complete: ${stats.nodeCount} nodes, ${stats.edgeCount} edges, ${stats.fileCount} files`);
                });
            }
            catch (error) {
                integration_core_1.eventBus.emit(integration_core_1.EventType.INDEX_FAILED, { error });
                vscode.window.showErrorMessage(`Reindex failed: ${error}`);
            }
        }
    });
    context.subscriptions.push(reindexCommand);
    // Show stats command
    const showStatsCommand = vscode.commands.registerCommand('spy-code.showStats', async () => {
        try {
            const stats = await cliBridge.getStats();
            const message = `
Index Statistics:
- Nodes: ${stats.nodeCount}
- Edges: ${stats.edgeCount}
- Files: ${stats.fileCount}
- Last Indexed: ${stats.lastIndexed || 'N/A'}
- Last Git SHA: ${stats.lastGitSha || 'N/A'}
        `.trim();
            vscode.window.showInformationMessage(message);
        }
        catch (error) {
            vscode.window.showErrorMessage(`Get stats failed: ${error}`);
        }
    });
    context.subscriptions.push(showStatsCommand);
    // Initialize command
    const initCommand = vscode.commands.registerCommand('spy-code.init', async () => {
        try {
            await cliBridge.init();
            vscode.window.showInformationMessage('Spy-Code initialized successfully');
        }
        catch (error) {
            vscode.window.showErrorMessage(`Init failed: ${error}`);
        }
    });
    context.subscriptions.push(initCommand);
}
function generateCallGraphText(node, callers, callees) {
    let text = `Call Graph for: ${node.name} (${node.kind})\n`;
    text += `Location: ${node.filePath}:${node.startLine}\n\n`;
    if (callers.length > 0) {
        text += 'Callers (functions that call this):\n';
        for (const edge of callers) {
            text += `  - ${edge.from.name} (${edge.from.kind}) in ${edge.from.filePath}:${edge.from.startLine}\n`;
        }
        text += '\n';
    }
    if (callees.length > 0) {
        text += 'Callees (functions called by this):\n';
        for (const edge of callees) {
            text += `  - ${edge.to.name} (${edge.to.kind}) in ${edge.to.filePath}:${edge.to.startLine}\n`;
        }
    }
    if (callers.length === 0 && callees.length === 0) {
        text += 'No call relationships found.\n';
    }
    return text;
}
//# sourceMappingURL=index.js.map