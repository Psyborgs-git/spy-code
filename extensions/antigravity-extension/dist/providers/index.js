"use strict";
/**
 * Language Providers - Register VS Code language features
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
exports.registerProviders = registerProviders;
const vscode = __importStar(require("vscode"));
function registerProviders(context, cliBridge, mcpClient) {
    // Register completion item provider
    const completionProvider = vscode.languages.registerCompletionItemProvider(['rust', 'python', 'typescript', 'javascript', 'go'], {
        async provideCompletionItems(document, position, token) {
            const line = document.lineAt(position.line);
            const word = line.text.split(/\s+/).find(w => position.character >= line.text.indexOf(w) &&
                position.character < line.text.indexOf(w) + w.length);
            if (!word || word.length < 3) {
                return [];
            }
            try {
                const results = await cliBridge.search(word, { limit: 5, kind: undefined });
                return results.map(r => {
                    const item = new vscode.CompletionItem(r.node.name, vscode.CompletionItemKind.Function);
                    item.detail = r.node.kind;
                    item.documentation = r.node.description;
                    item.insertText = r.node.name;
                    return item;
                });
            }
            catch (error) {
                return [];
            }
        }
    });
    context.subscriptions.push(completionProvider);
    // Register definition provider
    const definitionProvider = vscode.languages.registerDefinitionProvider(['rust', 'python', 'typescript', 'javascript', 'go'], {
        async provideDefinition(document, position, token) {
            const line = document.lineAt(position.line);
            const word = line.text.split(/\s+/).find(w => position.character >= line.text.indexOf(w) &&
                position.character < line.text.indexOf(w) + w.length);
            if (!word) {
                return [];
            }
            try {
                const results = await cliBridge.search(word, { limit: 1 });
                if (results.length > 0) {
                    const node = results[0].node;
                    const uri = vscode.Uri.file(node.filePath);
                    const range = new vscode.Range(node.startLine, 0, node.startLine, 0);
                    return [{
                            targetUri: uri,
                            targetRange: range,
                            originSelectionRange: new vscode.Range(position, position)
                        }];
                }
            }
            catch (error) {
                return [];
            }
            return [];
        }
    });
    context.subscriptions.push(definitionProvider);
    // Register reference provider
    const referenceProvider = vscode.languages.registerReferenceProvider(['rust', 'python', 'typescript', 'javascript', 'go'], {
        async provideReferences(document, position, context, token) {
            const line = document.lineAt(position.line);
            const word = line.text.split(/\s+/).find(w => position.character >= line.text.indexOf(w) &&
                position.character < line.text.indexOf(w) + w.length);
            if (!word) {
                return [];
            }
            try {
                const results = await cliBridge.search(word, { limit: 50 });
                return results.map(r => {
                    const uri = vscode.Uri.file(r.node.filePath);
                    const range = new vscode.Range(r.node.startLine, 0, r.node.startLine, 0);
                    return new vscode.Location(uri, range);
                });
            }
            catch (error) {
                return [];
            }
        }
    });
    context.subscriptions.push(referenceProvider);
    // Register hover provider
    const hoverProvider = vscode.languages.registerHoverProvider(['rust', 'python', 'typescript', 'javascript', 'go'], {
        async provideHover(document, position, token) {
            const line = document.lineAt(position.line);
            const word = line.text.split(/\s+/).find(w => position.character >= line.text.indexOf(w) &&
                position.character < line.text.indexOf(w) + w.length);
            if (!word) {
                return null;
            }
            try {
                const results = await cliBridge.search(word, { limit: 1 });
                if (results.length > 0) {
                    const node = results[0].node;
                    const markdown = new vscode.MarkdownString();
                    markdown.appendMarkdown(`**${node.name}** (${node.kind})\n\n`);
                    if (node.description) {
                        markdown.appendMarkdown(`${node.description}\n\n`);
                    }
                    if (node.signatures.length > 0) {
                        markdown.appendMarkdown('**Signatures:**\n');
                        for (const sig of node.signatures) {
                            const params = sig.params.map(p => `${p.name}: ${p.type || 'any'}`).join(', ');
                            markdown.appendMarkdown(`- \`${params}\` -> ${sig.returns || 'void'}\n`);
                        }
                    }
                    markdown.appendMarkdown(`\n*Location: ${node.filePath}:${node.startLine}*`);
                    return new vscode.Hover(markdown);
                }
            }
            catch (error) {
                return null;
            }
            return null;
        }
    });
    context.subscriptions.push(hoverProvider);
    // Register code lens provider
    const codeLensProvider = vscode.languages.registerCodeLensProvider(['rust', 'python', 'typescript', 'javascript', 'go'], {
        async provideCodeLenses(document, token) {
            const codeLenses = [];
            try {
                // Search for functions in the current file
                const filePath = document.uri.fsPath;
                const results = await cliBridge.search('', { limit: 100 });
                const fileResults = results.filter(r => r.node.filePath === filePath);
                for (const result of fileResults) {
                    if (result.node.kind === 'function') {
                        const range = new vscode.Range(result.node.startLine, 0, result.node.startLine, 0);
                        // Add "Show Callers" code lens
                        codeLenses.push(new vscode.CodeLens(range, {
                            title: 'Show Callers',
                            command: 'spy-code.showCallers',
                            arguments: [result.node.id]
                        }));
                        // Add "Show Callees" code lens
                        codeLenses.push(new vscode.CodeLens(range, {
                            title: 'Show Callees',
                            command: 'spy-code.showCallees',
                            arguments: [result.node.id]
                        }));
                    }
                }
            }
            catch (error) {
                // Ignore errors
            }
            return codeLenses;
        }
    });
    context.subscriptions.push(codeLensProvider);
}
//# sourceMappingURL=index.js.map