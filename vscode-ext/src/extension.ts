import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
    console.log('REC Lang extension active.');

    const diagnosticCollection = vscode.languages.createDiagnosticCollection('rec');
    context.subscriptions.push(diagnosticCollection);

    context.subscriptions.push(
        vscode.workspace.onDidOpenTextDocument((document) => {
            if (document.languageId === 'rec') {
                validateRecFile(document, diagnosticCollection);
            }
        })
    );

    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument((document) => {
            if (document.languageId === 'rec') {
                validateRecFile(document, diagnosticCollection);
            }
        })
    );

    context.subscriptions.push(
        vscode.workspace.onDidChangeTextDocument((event) => {
            if (event.document.languageId === 'rec') {
                validateRecFile(event.document, diagnosticCollection);
            }
        })
    );

    const hoverProvider = vscode.languages.registerHoverProvider('rec', {
        provideHover(document, position, token) {
            const range = document.getWordRangeAtPosition(position);
            const word = document.getText(range);
            const line = document.lineAt(position.line).text;
            const linePrefix = line.substring(0, position.character);
            
            const typeInfo: { [key: string]: { syntax: string, description: string } } = {
                'string': {
                    syntax: 'string',
                    description: 'A text string value. Must be enclosed in double quotes.\n\nExample: `"hello world"`'
                },
                'int': {
                    syntax: 'int',
                    description: 'An integer number (positive or negative).\n\nExample: `42`, `-100`'
                },
                'float': {
                    syntax: 'float',
                    description: 'A floating point number.\n\nExample: `3.14`, `-0.5`'
                },
                'bool': {
                    syntax: 'bool',
                    description: 'A boolean value.\n\nValid values: `true`, `false`'
                },
                'url': {
                    syntax: 'url("...")',
                    description: 'An HTTP/HTTPS URL. Must be valid and enclosed in quotes.\n\nExample: `url("https://api.example.com")`'
                },
                'socket': {
                    syntax: 'socket("...")',
                    description: 'An IPv4 socket address (IP:port). Must be valid format.\n\nExample: `socket("127.0.0.1:8080")`'
                },
                'pubkey': {
                    syntax: 'pubkey("...")',
                    description: 'A Base58 encoded ed25519 public key (32 bytes) for Solana.\n\nExample: `pubkey("DRpbCBMxVnDK7maPM5tGv6MvB3v1sRMC86PZ8okm21hy")`'
                }
            };

            if (typeInfo[word]) {
                const info = typeInfo[word];
                const markdown = new vscode.MarkdownString();
                markdown.appendCodeblock(info.syntax, 'rec');
                markdown.appendMarkdown(info.description);
                return new vscode.Hover(markdown);
            }

            if (word === 'enum' && linePrefix.includes('@')) {
                const markdown = new vscode.MarkdownString();
                markdown.appendMarkdown('**Enum Definition**\n\n');
                markdown.appendMarkdown('Defines a type-safe enumeration with variants.\n\n');
                markdown.appendMarkdown('**Syntax:**\n');
                markdown.appendCodeblock(`@enum Name {
    // Unit variants
    VARIANT1
    VARIANT2
    
    // Tuple variants
    VARIANT3(type1, type2)
    
    // Struct variants
    VARIANT4 { field1: type1, field2: type2 }
}`, 'rec');
                markdown.appendMarkdown('\n**Example:**\n');
                markdown.appendCodeblock(`@enum Database {
    Postgres { host: string, port: int, ssl: bool }
    Redis { host: string, port: int }
    InMemory
}`, 'rec');
                return new vscode.Hover(markdown);
            }

            if (word === 'type' && linePrefix.includes('@')) {
                const markdown = new vscode.MarkdownString();
                markdown.appendMarkdown('**Type Definition**\n\n');
                markdown.appendMarkdown('Defines a reusable struct type with typed fields.\n\n');
                markdown.appendMarkdown('**Syntax:**\n');
                markdown.appendCodeblock(`@type Name {
    field1: type
    field2?: type  // Optional field
}`, 'rec');
                markdown.appendMarkdown('\n**Example:**\n');
                markdown.appendCodeblock(`@type ServerConfig {
    host: string
    port: int
    ssl_enabled: bool
    ssl_cert?: string  // Optional
}`, 'rec');
                return new vscode.Hover(markdown);
            }

            if (word === 'include' && linePrefix.includes('#')) {
                const markdown = new vscode.MarkdownString();
                markdown.appendMarkdown('**Include Statement**\n\n');
                markdown.appendMarkdown('Includes another REC file at this position.\n\n');
                markdown.appendMarkdown('**Syntax:**\n');
                markdown.appendCodeblock('#include "path/to/file.rec"', 'rec');
                markdown.appendMarkdown('\n**Note:** Included files are merged at parse time.');
                return new vscode.Hover(markdown);
            }

            if (word === '[' || (range && document.getText(new vscode.Range(range.start, range.end.translate(0, 1))) === '[]')) {
                const markdown = new vscode.MarkdownString();
                markdown.appendMarkdown('**Array Type**\n\n');
                markdown.appendMarkdown('Arrays contain multiple values of the same type.\n\n');
                markdown.appendMarkdown('**Syntax:**\n');
                markdown.appendCodeblock('[type]  // In type definitions\n[value1, value2, ...]  // In values', 'rec');
                markdown.appendMarkdown('\n**Example:**\n');
                markdown.appendCodeblock(`origins: [string]  // Type definition
origins: ["http://localhost:3000", "https://api.example.com"]`, 'rec');
                return new vscode.Hover(markdown);
            }

            if (word === 'true' || word === 'false') {
                const markdown = new vscode.MarkdownString();
                markdown.appendMarkdown(`**Boolean Value**\n\n\`${word}\` is a boolean literal.`);
                return new vscode.Hover(markdown);
            }

            if (word === 'null') {
                const markdown = new vscode.MarkdownString();
                markdown.appendMarkdown('**Null Value**\n\nRepresents an absent or undefined value.\n\nTypically used for optional fields.');
                return new vscode.Hover(markdown);
            }

            const functions: { [key: string]: { signature: string, description: string } } = {
                'url': {
                    signature: 'url(address: string) -> url',
                    description: 'Creates a validated URL value. The URL must be a valid HTTP or HTTPS address.'
                },
                'socket': {
                    signature: 'socket(address: string) -> socket',
                    description: 'Creates a validated IPv4 socket address. Format must be "IP:port".'
                },
                'pubkey': {
                    signature: 'pubkey(key: string) -> pubkey',
                    description: 'Creates a validated Solana public key. Must be a valid Base58 encoded 32-byte ed25519 key.'
                }
            };

            if (functions[word] && line.includes(word + '(')) {
                const func = functions[word];
                const markdown = new vscode.MarkdownString();
                markdown.appendCodeblock(func.signature, 'rec');
                markdown.appendMarkdown(func.description);
                return new vscode.Hover(markdown);
            }

            if (word === '?' || (range && position.character > 0 && line[position.character - 1] === '?')) {
                const markdown = new vscode.MarkdownString();
                markdown.appendMarkdown('**Optional Field**\n\n');
                markdown.appendMarkdown('The `?` suffix marks a field as optional in type definitions.\n\n');
                markdown.appendMarkdown('**Example:**\n');
                markdown.appendCodeblock('password?: string  // This field can be omitted', 'rec');
                return new vscode.Hover(markdown);
            }

            if (line.includes('.') && /[A-Z][a-zA-Z0-9_]*\.[A-Z][a-zA-Z0-9_]*/.test(line)) {
                const enumMatch = line.match(/([A-Z][a-zA-Z0-9_]*)\.([A-Z][a-zA-Z0-9_]*)/);
                if (enumMatch && (word === enumMatch[1] || word === enumMatch[2])) {
                    const markdown = new vscode.MarkdownString();
                    markdown.appendMarkdown('**Enum Variant Usage**\n\n');
                    markdown.appendMarkdown('References a specific variant of an enum type.\n\n');
                    markdown.appendMarkdown('**Syntax:**\n');
                    markdown.appendCodeblock('EnumName.VariantName\nEnumName.VariantName { fields }\nEnumName.VariantName(values)', 'rec');
                    return new vscode.Hover(markdown);
                }
            }

            return null;
        }
    });

    context.subscriptions.push(hoverProvider);

    const completionProvider = vscode.languages.registerCompletionItemProvider('rec', {
        provideCompletionItems(document, position, token, context) {
            const linePrefix = document.lineAt(position).text.substr(0, position.character);
            const completionItems: vscode.CompletionItem[] = [];

            if (linePrefix.trim() === '' || linePrefix.trim() === '@') {
                const enumItem = new vscode.CompletionItem('@enum', vscode.CompletionItemKind.Keyword);
                enumItem.insertText = new vscode.SnippetString('@enum ${1:Name} {\n\t${2:VARIANT1}\n\t${3:VARIANT2}\n}');
                enumItem.documentation = 'Define a new enum type';
                completionItems.push(enumItem);

                const typeItem = new vscode.CompletionItem('@type', vscode.CompletionItemKind.Keyword);
                typeItem.insertText = new vscode.SnippetString('@type ${1:Name} {\n\t${2:field}: ${3:string}\n}');
                typeItem.documentation = 'Define a new struct type';
                completionItems.push(typeItem);
            }

            if (linePrefix.trim() === '' || linePrefix.trim() === '#') {
                const includeItem = new vscode.CompletionItem('#include', vscode.CompletionItemKind.Keyword);
                includeItem.insertText = new vscode.SnippetString('#include "${1:path/to/file.rec}"');
                includeItem.documentation = 'Include another REC file';
                completionItems.push(includeItem);
            }

            if (linePrefix.endsWith(': ') || linePrefix.match(/:\s*$/)) {
                const types = ['string', 'int', 'float', 'bool', 'url', 'socket', 'pubkey'];
                types.forEach(type => {
                    const item = new vscode.CompletionItem(type, vscode.CompletionItemKind.TypeParameter);
                    item.documentation = `Built-in type: ${type}`;
                    completionItems.push(item);
                });

                const arrayItem = new vscode.CompletionItem('[type]', vscode.CompletionItemKind.TypeParameter);
                arrayItem.insertText = new vscode.SnippetString('[${1:string}]');
                arrayItem.documentation = 'Array type';
                completionItems.push(arrayItem);
            }

            if (linePrefix.endsWith(': ')) {
                ['true', 'false'].forEach(value => {
                    const item = new vscode.CompletionItem(value, vscode.CompletionItemKind.Value);
                    item.documentation = `Boolean value: ${value}`;
                    completionItems.push(item);
                });

                const nullItem = new vscode.CompletionItem('null', vscode.CompletionItemKind.Value);
                nullItem.documentation = 'Null value';
                completionItems.push(nullItem);
            }

            if (linePrefix.endsWith(': ') || linePrefix.match(/:\s+$/)) {
                const urlItem = new vscode.CompletionItem('url(...)', vscode.CompletionItemKind.Function);
                urlItem.insertText = new vscode.SnippetString('url("${1:https://example.com}")');
                urlItem.documentation = 'Create a URL value';
                completionItems.push(urlItem);

                const socketItem = new vscode.CompletionItem('socket(...)', vscode.CompletionItemKind.Function);
                socketItem.insertText = new vscode.SnippetString('socket("${1:127.0.0.1:8080}")');
                socketItem.documentation = 'Create a socket address value';
                completionItems.push(socketItem);

                const pubkeyItem = new vscode.CompletionItem('pubkey(...)', vscode.CompletionItemKind.Function);
                pubkeyItem.insertText = new vscode.SnippetString('pubkey("${1:base58pubkey}")');
                pubkeyItem.documentation = 'Create a Solana public key value';
                completionItems.push(pubkeyItem);
            }

            return completionItems;
        }
    }, ' ', ':', '@', '#');

    context.subscriptions.push(completionProvider);

    const symbolProvider = vscode.languages.registerDocumentSymbolProvider('rec', {
        provideDocumentSymbols(document, token) {
            const symbols: vscode.DocumentSymbol[] = [];
            const text = document.getText();
            const lines = text.split('\n');

            for (let i = 0; i < lines.length; i++) {
                const line = lines[i];

                const enumMatch = line.match(/@enum\s+([A-Z][a-zA-Z0-9_]*)/);
                if (enumMatch) {
                    const name = enumMatch[1];
                    const startPos = new vscode.Position(i, 0);
                    
                    let endLine = i;
                    let braceCount = 0;
                    for (let j = i; j < lines.length; j++) {
                        if (lines[j].includes('{')) braceCount++;
                        if (lines[j].includes('}')) braceCount--;
                        if (braceCount === 0 && lines[j].includes('}')) {
                            endLine = j;
                            break;
                        }
                    }
                    
                    const endPos = new vscode.Position(endLine, lines[endLine].length);
                    const symbol = new vscode.DocumentSymbol(
                        name,
                        'enum',
                        vscode.SymbolKind.Enum,
                        new vscode.Range(startPos, endPos),
                        new vscode.Range(startPos, endPos)
                    );
                    symbols.push(symbol);
                }

                const typeMatch = line.match(/@type\s+([A-Z][a-zA-Z0-9_]*)/);
                if (typeMatch) {
                    const name = typeMatch[1];
                    const startPos = new vscode.Position(i, 0);
                    
                    let endLine = i;
                    let braceCount = 0;
                    for (let j = i; j < lines.length; j++) {
                        if (lines[j].includes('{')) braceCount++;
                        if (lines[j].includes('}')) braceCount--;
                        if (braceCount === 0 && lines[j].includes('}')) {
                            endLine = j;
                            break;
                        }
                    }
                    
                    const endPos = new vscode.Position(endLine, lines[endLine].length);
                    const symbol = new vscode.DocumentSymbol(
                        name,
                        'type',
                        vscode.SymbolKind.Struct,
                        new vscode.Range(startPos, endPos),
                        new vscode.Range(startPos, endPos)
                    );
                    symbols.push(symbol);
                }

                const keyMatch = line.match(/^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*:/);
                if (keyMatch && !line.includes('@') && !line.includes('#')) {
                    const name = keyMatch[1];
                    const startPos = new vscode.Position(i, line.indexOf(name));
                    const endPos = new vscode.Position(i, line.length);
                    
                    const symbol = new vscode.DocumentSymbol(
                        name,
                        'property',
                        vscode.SymbolKind.Property,
                        new vscode.Range(startPos, endPos),
                        new vscode.Range(startPos, endPos)
                    );
                    symbols.push(symbol);
                }
            }

            return symbols;
        }
    });

    context.subscriptions.push(symbolProvider);

    // formatting provider
    const formattingProvider = vscode.languages.registerDocumentFormattingEditProvider('rec', {
        provideDocumentFormattingEdits(document, options, token) {
            const edits: vscode.TextEdit[] = [];
            const text = document.getText();
            const lines = text.split('\n');
            
            let indentLevel = 0;
            const indentStr = options.insertSpaces ? ' '.repeat(options.tabSize) : '\t';
            
            for (let i = 0; i < lines.length; i++) {
                const line = lines[i].trim();
                
                if (line === '') continue;
                
                if (line.startsWith('}')) {
                    indentLevel = Math.max(0, indentLevel - 1);
                }
                
                const newLine = indentStr.repeat(indentLevel) + line;
                
                if (newLine !== lines[i]) {
                    edits.push(vscode.TextEdit.replace(
                        new vscode.Range(i, 0, i, lines[i].length),
                        newLine
                    ));
                }
                
                if (line.endsWith('{')) {
                    indentLevel++;
                }
            }
            
            return edits;
        }
    });

    context.subscriptions.push(formattingProvider);

    const convertToJsonCommand = vscode.commands.registerCommand('rec.convertToJson', () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'rec') {
            vscode.window.showErrorMessage('Please open a REC file first');
            return;
        }

        // TODO: run the CLI here
        vscode.window.showInformationMessage('');
    });

    context.subscriptions.push(convertToJsonCommand);
}

function validateRecFile(document: vscode.TextDocument, diagnosticCollection: vscode.DiagnosticCollection) {
    const diagnostics: vscode.Diagnostic[] = [];
    const text = document.getText();
    const lines = text.split('\n');

    let braceBalance = 0;
    let inString = false;
    let inComment = false;
    let inBlockComment = false;

    for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        
        if (line.trim() === '' || line.trim().startsWith('//')) {
            continue;
        }

        if (line.includes('/*')) {
            inBlockComment = true;
        }
        if (line.includes('*/')) {
            inBlockComment = false;
            continue;
        }
        if (inBlockComment) {
            continue;
        }

        const socketMatch = line.match(/socket\s*\(\s*([^)]+)\s*\)/);
        if (socketMatch && !socketMatch[1].includes('"')) {
            const startIndex = line.indexOf('socket(');
            const diagnostic = new vscode.Diagnostic(
                new vscode.Range(i, startIndex, i, startIndex + socketMatch[0].length),
                'Socket addresses must be quoted strings: socket("127.0.0.1:8080")',
                vscode.DiagnosticSeverity.Error
            );
            diagnostics.push(diagnostic);
        }

        const urlMatch = line.match(/url\s*\(\s*([^)]+)\s*\)/);
        if (urlMatch && !urlMatch[1].includes('"')) {
            const startIndex = line.indexOf('url(');
            const diagnostic = new vscode.Diagnostic(
                new vscode.Range(i, startIndex, i, startIndex + urlMatch[0].length),
                'URLs must be quoted strings: url("https://example.com")',
                vscode.DiagnosticSeverity.Error
            );
            diagnostics.push(diagnostic);
        }

        const pubkeyMatch = line.match(/pubkey\s*\(\s*([^)]+)\s*\)/);
        if (pubkeyMatch && !pubkeyMatch[1].includes('"')) {
            const startIndex = line.indexOf('pubkey(');
            const diagnostic = new vscode.Diagnostic(
                new vscode.Range(i, startIndex, i, startIndex + pubkeyMatch[0].length),
                'Public keys must be quoted strings: pubkey("base58key...")',
                vscode.DiagnosticSeverity.Error
            );
            diagnostics.push(diagnostic);
        }

        let charInString = false;
        let escapeNext = false;
        
        for (let j = 0; j < line.length; j++) {
            const char = line[j];
            
            if (escapeNext) {
                escapeNext = false;
                continue;
            }
            
            if (char === '\\') {
                escapeNext = true;
                continue;
            }
            
            if (char === '"' && !charInString) {
                charInString = true;
            } else if (char === '"' && charInString) {
                charInString = false;
            }
            
            if (!charInString) {
                if (char === '{') braceBalance++;
                if (char === '}') braceBalance--;
            }
        }
    }

    if (braceBalance !== 0) {
        const lastLine = lines.length - 1;
        const diagnostic = new vscode.Diagnostic(
            new vscode.Range(lastLine, 0, lastLine, lines[lastLine].length),
            `Unmatched braces in document: ${braceBalance > 0 ? braceBalance + ' unclosed {' : Math.abs(braceBalance) + ' extra }'}`,
            vscode.DiagnosticSeverity.Error
        );
        diagnostics.push(diagnostic);
    }

    diagnosticCollection.set(document.uri, diagnostics);
}

export function deactivate() {}