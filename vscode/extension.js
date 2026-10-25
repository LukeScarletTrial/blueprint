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
    { label: 'C.Include', kind: vscode.CompletionItemKind.Function, detail: 'Include C header for FFI', snippet: 'C.Include("${1:math.h}")' },
    { label: 'C.Link', kind: vscode.CompletionItemKind.Function, detail: 'Link external native library', snippet: 'C.Link("${1:m}")' },
    { label: 'C.Extern', kind: vscode.CompletionItemKind.Function, detail: 'Declare external C function signature', snippet: 'C.Extern("${1:double sqrt(double x)}")' },
    { label: 'file_read', kind: vscode.CompletionItemKind.Function, detail: 'Read file contents to string', snippet: 'file_read("${1:path}")' },
    { label: 'file_write', kind: vscode.CompletionItemKind.Function, detail: 'Write string data to file', snippet: 'file_write("${1:path}", ${2:data})' },
    { label: 'file_exists', kind: vscode.CompletionItemKind.Function, detail: 'Check if file exists', snippet: 'file_exists("${1:path}")' },
    { label: 'path_join', kind: vscode.CompletionItemKind.Function, detail: 'Join path segments', snippet: 'path_join(${1:a}, ${2:b})' },
    { label: 'path_ext', kind: vscode.CompletionItemKind.Function, detail: 'Get file extension', snippet: 'path_ext(${1:path})' },
    { label: 'path_filename', kind: vscode.CompletionItemKind.Function, detail: 'Get path file name', snippet: 'path_filename(${1:path})' },
    { label: 'env_get', kind: vscode.CompletionItemKind.Function, detail: 'Get environment variable', snippet: 'env_get("${1:KEY}")' },
    { label: 'env_set', kind: vscode.CompletionItemKind.Function, detail: 'Set environment variable', snippet: 'env_set("${1:KEY}", ${2:val})' },
    { label: 'time_now', kind: vscode.CompletionItemKind.Function, detail: 'Get epoch timestamp in seconds', snippet: 'time_now()' },
    { label: 'time_sleep', kind: vscode.CompletionItemKind.Function, detail: 'Sleep thread for milliseconds', snippet: 'time_sleep(${1:ms})' },
    { label: 'str_to_int', kind: vscode.CompletionItemKind.Function, detail: 'Convert string to integer', snippet: 'str_to_int(${1:str})' },
    { label: 'str_to_float', kind: vscode.CompletionItemKind.Function, detail: 'Convert string to float', snippet: 'str_to_float(${1:str})' },
    { label: 'str_contains', kind: vscode.CompletionItemKind.Function, detail: 'Check if string contains substring', snippet: 'str_contains(${1:haystack}, ${2:needle})' },
    { label: 'str_starts_with', kind: vscode.CompletionItemKind.Function, detail: 'Check if string starts with prefix', snippet: 'str_starts_with(${1:str}, ${2:prefix})' },
    { label: 'str_ends_with', kind: vscode.CompletionItemKind.Function, detail: 'Check if string ends with suffix', snippet: 'str_ends_with(${1:str}, ${2:suffix})' },
    { label: 'math_sqrt', kind: vscode.CompletionItemKind.Function, detail: 'Square root', snippet: 'math_sqrt(${1:x})' },
    { label: 'math_floor', kind: vscode.CompletionItemKind.Function, detail: 'Floor number', snippet: 'math_floor(${1:x})' },
    { label: 'math_ceil', kind: vscode.CompletionItemKind.Function, detail: 'Ceil number', snippet: 'math_ceil(${1:x})' },
    { label: 'math_round', kind: vscode.CompletionItemKind.Function, detail: 'Round number', snippet: 'math_round(${1:x})' },
    { label: 'int', kind: vscode.CompletionItemKind.TypeParameter, detail: 'Primitive integer type (64-bit)' },
    { label: 'i8', kind: vscode.CompletionItemKind.TypeParameter, detail: '8-bit signed integer' },
    { label: 'i16', kind: vscode.CompletionItemKind.TypeParameter, detail: '16-bit signed integer' },
    { label: 'i32', kind: vscode.CompletionItemKind.TypeParameter, detail: '32-bit signed integer' },
    { label: 'i64', kind: vscode.CompletionItemKind.TypeParameter, detail: '64-bit signed integer' },
    { label: 'u8', kind: vscode.CompletionItemKind.TypeParameter, detail: '8-bit unsigned integer' },
    { label: 'u16', kind: vscode.CompletionItemKind.TypeParameter, detail: '16-bit unsigned integer' },
    { label: 'u32', kind: vscode.CompletionItemKind.TypeParameter, detail: '32-bit unsigned integer' },
    { label: 'u64', kind: vscode.CompletionItemKind.TypeParameter, detail: '64-bit unsigned integer' },
    { label: 'f32', kind: vscode.CompletionItemKind.TypeParameter, detail: '32-bit floating point' },
    { label: 'f64', kind: vscode.CompletionItemKind.TypeParameter, detail: '64-bit floating point' },
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
        ['blueprint', 'blueprint-module'],
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
