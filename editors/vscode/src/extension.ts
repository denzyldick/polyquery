import * as vscode from 'vscode';
import { exec, ChildProcess } from 'child_process';
import * as path from 'path';

let client: PolyqueryClient | undefined;

class PolyqueryClient {
    private process: ChildProcess;
    private outputChannel: vscode.OutputChannel;

    constructor(serverPath: string) {
        this.outputChannel = vscode.window.createOutputChannel('Polyquery');
        const databaseUrl = vscode.workspace.getConfiguration('polyquery').get<string>('databaseUrl') || '';

        const env: NodeJS.ProcessEnv = { ...process.env };
        if (databaseUrl) {
            env.POLYQUERY_DATABASE_URL = databaseUrl;
        }

        this.process = exec(serverPath, { env }, (err, stdout, stderr) => {
            if (stderr) {
                this.outputChannel.appendLine(stderr);
            }
        });

        this.outputChannel.appendLine('Polyquery LSP server started');
        this.outputChannel.show();
    }

    dispose() {
        this.process.kill();
        this.outputChannel.dispose();
    }
}

export function activate(context: vscode.ExtensionContext) {
    const serverPath = vscode.workspace.getConfiguration('polyquery').get<string>('serverPath') || 'polyquery';

    client = new PolyqueryClient(serverPath);

    context.subscriptions.push(
        vscode.commands.registerCommand('polyquery.runQuery', async (uri?: string, sql?: string) => {
            const editor = vscode.window.activeTextEditor;
            if (!editor) return;

            const document = editor.document;
            const selection = editor.selection;
            const text = sql || (selection.isEmpty ? document.getText() : document.getText(selection));

            if (!text.trim()) {
                vscode.window.showWarningMessage('No SQL selected');
                return;
            }

            const databaseUrl = vscode.workspace.getConfiguration('polyquery').get<string>('databaseUrl');
            if (!databaseUrl) {
                const input = await vscode.window.showInputBox({
                    prompt: 'Enter database URL (e.g., postgresql://user:pass@localhost/db)',
                    placeHolder: 'postgresql://user:pass@localhost/mydb',
                    password: true,
                });
                if (input) {
                    await vscode.workspace.getConfiguration('polyquery').update('databaseUrl', input, true);
                    vscode.window.showInformationMessage('Database URL saved. Please reload window to connect.');
                }
                return;
            }

            const outputChannel = vscode.window.createOutputChannel('Polyquery Results');
            outputChannel.clear();
            outputChannel.appendLine(`Running SQL...\n`);
            outputChannel.appendLine(text);
            outputChannel.appendLine('');

            // Send to LSP server via workspace/executeCommand
            try {
                const result = await vscode.commands.executeCommand<{ type: string; text?: string; message?: string }>(
                    'workspace.executeCommand',
                    'polyquery.runQuery',
                    [uri || document.uri.toString(), text]
                );

                if (result) {
                    if (result.type === 'error') {
                        outputChannel.appendLine(`Error: ${result.message}`);
                    } else if (result.text) {
                        outputChannel.appendLine(result.text);
                    }
                }
            } catch (err: any) {
                outputChannel.appendLine(`Error: ${err.message || err}`);
            }

            outputChannel.show();
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('polyquery.setDatabaseUrl', async () => {
            const input = await vscode.window.showInputBox({
                prompt: 'Enter database URL',
                placeHolder: 'postgresql://user:pass@localhost/mydb',
                password: true,
            });
            if (input) {
                await vscode.workspace.getConfiguration('polyquery').update('databaseUrl', input, true);
                vscode.window.showInformationMessage('Database URL saved. Please reload window to connect.');
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('polyquery.clearDatabaseUrl', async () => {
            await vscode.workspace.getConfiguration('polyquery').update('databaseUrl', undefined, true);
            vscode.window.showInformationMessage('Database URL cleared. Please reload window.');
        })
    );

    vscode.window.showInformationMessage('Polyquery activated');
}

export function deactivate() {
    if (client) {
        client.dispose();
        client = undefined;
    }
}
