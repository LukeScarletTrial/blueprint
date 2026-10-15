const vscode = require('vscode');

/**
 * Regex patterns to extract declarations across documents
 */
const FUNC_REGEX = /func\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)(?:\s*:\s*([A-Za-z0-9_]+))?/g;
const STRUCT_REGEX = /defstruct\s+([A-Za-z_][A-Za-z0-9_]*)\s*\[([^\]]*)\]/g;
const VAR_REGEX = /def\s+([A-Za-z_][A-Za-z0-9_]*)(?:\s*:\s*([A-Za-z0-9_]+))?\s*=/g;

/**
 * Built-in Blueprint keywords & types
 */
const BUILTINS = [
    { label: 'output', kind: vscode.CompletionItemKind.Function, detail: 'Built-in output function', snippet: 'output(${1:value})' },
    { label: 'input', kind: vscode.CompletionItemKind.Function, detail: 'Read line from stdin', snippet: 'input("${1:prompt: }")' },
    { label: 'str_len', kind: vscode.CompletionItemKind.Function, detail: 'Get string length', snippet: 'str_len(${1:str})' },
    { label: 'math_abs', kind: vscode.CompletionItemKind.Function, detail: 'Absolute value', snippet: 'math_abs(${1:n})' },
    { label: 'math_pow', kind: vscode.CompletionItemKind.Function, detail: 'Power calculation', snippet: 'math_pow(${1:base}, ${2:exp})' },
    { label: 'math_min', kind: vscode.CompletionItemKind.Function, detail: 'Minimum of two numbers', snippet: 'math_min(${1:a}, ${2:b})' },
    { label: 'math_max', kind: vscode.CompletionItemKind.Function, detail: 'Maximum of two numbers', snippet: 'math_max(${1:a}, ${2:b})' },
    { label: 'window_open', kind: vscode.CompletionItemKind.Function, detail: 'Open 2D graphics window', snippet: 'window_open("${1:Window Title}", ${2:800}, ${3:600})' },
    { label: 'window_close', kind: vscode.CompletionItemKind.Function, detail: 'Close graphics window', snippet: 'window_close()' },
    { label: 'window_should_close', kind: vscode.CompletionItemKind.Function, detail: 'Check if window close was requested', snippet: 'window_should_close()' },
    { label: 'window_poll', kind: vscode.CompletionItemKind.Function, detail: 'Poll window events', snippet: 'window_poll()' },
    { label: 'window_present', kind: vscode.CompletionItemKind.Function, detail: 'Present backbuffer to screen', snippet: 'window_present()' },
    { label: 'draw_clear', kind: vscode.CompletionItemKind.Function, detail: 'Clear canvas with RGB color', snippet: 'draw_clear(${1:r}, ${2:g}, ${3:b})' },
    { label: 'draw_rect', kind: vscode.CompletionItemKind.Function, detail: 'Draw filled rectangle', snippet: 'draw_rect(${1:x}, ${2:y}, ${3:w}, ${4:h}, ${5:r}, ${6:g}, ${7:b})' },
    { label: 'draw_circle', kind: vscode.CompletionItemKind.Function, detail: 'Draw circle', snippet: 'draw_circle(${1:cx}, ${2:cy}, ${3:radius}, ${4:r}, ${5:g}, ${6:b})' },
    { label: 'draw_line', kind: vscode.CompletionItemKind.Function, detail: 'Draw line between points', snippet: 'draw_line(${1:x1}, ${2:y1}, ${3:x2}, ${4:y2}, ${5:r}, ${6:g}, ${7:b})' },
    { label: 'draw_text', kind: vscode.CompletionItemKind.Function, detail: 'Draw text string', snippet: 'draw_text("${1:text}", ${2:x}, ${3:y}, ${4:size}, ${5:r}, ${6:g}, ${7:b})' },
    { label: 'is_key_down', kind: vscode.CompletionItemKind.Function, detail: 'Check if virtual key is down', snippet: 'is_key_down(${1:keyCode})' },
    { label: 'Blueprint.Setup', kind: vscode.CompletionItemKind.Function, detail: 'Initialize Blueprint app metadata', snippet: 'Blueprint.Setup("${1:App Name}", ${2:true})' },
    { label: 'int', kind: vscode.CompletionItemKind.TypeParameter, detail: 'Primitive integer type' },
    { label: 'string', kind: vscode.CompletionItemKind.TypeParameter, detail: 'Primitive string type' },
    { label: 'str', kind: vscode.CompletionItemKind.TypeParameter, detail: 'Alias for string type' },
    { label: 'bool', kind: vscode.CompletionItemKind.TypeParameter, detail: 'Primitive boolean type' },
    { label: 'void', kind: vscode.CompletionItemKind.TypeParameter, detail: 'Void return type' },
    { label: 'true', kind: vscode.CompletionItemKind.Keyword, detail: 'Boolean true' },
    { label: 'false', kind: vscode.CompletionItemKind.Keyword, detail: 'Boolean false' },
    { label: 'return', kind: vscode.CompletionItemKind.Keyword, detail: 'Return statement', snippet: 'return ${1:value}' },
    { label: 'if', kind: vscode.CompletionItemKind.Keyword, detail: 'If block', snippet: 'if ${1:condition} [\n\t${2}\n]' },
    { label: 'ifelse', kind: vscode.CompletionItemKind.Snippet, detail: 'If-Else block', snippet: 'if ${1:condition} [\n\t${2}\n] else [\n\t${3}\n]' },
    { label: 'func', kind: vscode.CompletionItemKind.Snippet, detail: 'Function declaration', snippet: 'func ${1:Name}(${2:params}): ${3:void} [\n\t${0}\n]' },
    { label: 'defstruct', kind: vscode.CompletionItemKind.Snippet, detail: 'Struct declaration', snippet: 'defstruct ${1:Name} [\n\t${2:field}: ${3:type}\n]' },
    { label: 'def', kind: vscode.CompletionItemKind.Snippet, detail: 'Variable declaration', snippet: 'def ${1:name} = ${2:value}' }
];

function activate(context) {
    const provider = vscode.languages.registerCompletionItemProvider(
        'blueprint',
        {
            provideCompletionItems(document, position) {
                const items = [];
                const linePrefix = document.lineAt(position).text.substr(0, position.character);

                // Add built-ins
                for (const b of BUILTINS) {
                    const item = new vscode.CompletionItem(b.label, b.kind);
                    item.detail = b.detail;
                    if (b.snippet) {
                        item.insertText = new vscode.SnippetString(b.snippet);
                    }
                    items.push(item);
                }

                // Scan all open Blueprint documents and current file
                const text = document.getText();
                const seen = new Set();

                // 1. Functions with arguments
                let match;
                FUNC_REGEX.lastIndex = 0;
                while ((match = FUNC_REGEX.exec(text)) !== null) {
                    const funcName = match[1];
                    const rawParams = match[2].trim();
                    const returnTy = match[3] || 'void';

                    if (seen.has('func_' + funcName)) continue;
                    seen.add('func_' + funcName);

                    const item = new vscode.CompletionItem(funcName, vscode.CompletionItemKind.Function);
                    item.detail = `func ${funcName}(${rawParams}): ${returnTy}`;

                    // Build parameter placeholders for call
                    if (rawParams.length === 0) {
                        item.insertText = new vscode.SnippetString(`${funcName}()`);
                    } else {
                        const params = rawParams.split(',').map(p => p.trim());
                        const snippetArgs = params.map((p, idx) => `\${${idx + 1}:${p}}`).join(', ');
                        item.insertText = new vscode.SnippetString(`${funcName}(${snippetArgs})`);
                    }
                    item.documentation = new vscode.MarkdownString(`Call function \`${funcName}\`\n\nParameters: \`${rawParams || 'none'}\`\nReturns: \`${returnTy}\``);
                    items.push(item);
                }

                // 2. Structs with braced initialization snippet
                STRUCT_REGEX.lastIndex = 0;
                while ((match = STRUCT_REGEX.exec(text)) !== null) {
                    const structName = match[1];
                    const rawFields = match[2].trim();

                    if (seen.has('struct_' + structName)) continue;
                    seen.add('struct_' + structName);

                    const item = new vscode.CompletionItem(structName, vscode.CompletionItemKind.Struct);
                    item.detail = `defstruct ${structName}`;

                    // Struct init snippet: StructName { field: ${1:val}, field2: ${2:val} }
                    const fieldLines = rawFields.split(',').map(f => f.trim()).filter(f => f.length > 0);
                    if (fieldLines.length > 0) {
                        const snippetFields = fieldLines.map((f, idx) => {
                            const parts = f.split(':').map(p => p.trim());
                            const fname = parts[0];
                            const ftype = parts[1] || 'val';
                            return `${fname}: \${${idx + 1}:${ftype}}`;
                        }).join(', ');
                        item.insertText = new vscode.SnippetString(`${structName} { ${snippetFields} }`);
                    } else {
                        item.insertText = new vscode.SnippetString(`${structName} { }`);
                    }
                    item.documentation = new vscode.MarkdownString(`Initialize struct \`${structName}\` with named fields.`);
                    items.push(item);
                }

                // 3. Variables
                VAR_REGEX.lastIndex = 0;
                while ((match = VAR_REGEX.exec(text)) !== null) {
                    const varName = match[1];
                    const varTy = match[2] || '';

                    if (seen.has('var_' + varName)) continue;
                    seen.add('var_' + varName);

                    const item = new vscode.CompletionItem(varName, vscode.CompletionItemKind.Variable);
                    item.detail = varTy ? `def ${varName}: ${varTy}` : `def ${varName}`;
                    items.push(item);
                }

                return items;
            }
        },
        '.', '(', ':', ' ' // Trigger characters
    );

    context.subscriptions.push(provider);
}

function deactivate() {}

module.exports = {
    activate,
    deactivate
};
