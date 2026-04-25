const vscode = require("vscode");

const DIAGNOSTIC_SOURCE = "Harper Review";
const suggestionStore = new Map();

function activate(context) {
    const diagnostics = vscode.languages.createDiagnosticCollection("harperReview");
    const output = vscode.window.createOutputChannel("Harper Review");
    const statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);

    statusBarItem.text = "$(search-view-icon) Harper Review";
    statusBarItem.tooltip = "Review the active file with Harper";
    statusBarItem.command = "harperReview.reviewCurrentFile";
    statusBarItem.show();

    context.subscriptions.push(diagnostics, output, statusBarItem);

    context.subscriptions.push(
        vscode.commands.registerCommand("harperReview.reviewCurrentFile", async () => {
            await reviewActiveEditor({
                selectionOnly: false,
                diagnostics,
                output,
                statusBarItem
            });
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand("harperReview.reviewSelection", async () => {
            await reviewActiveEditor({
                selectionOnly: true,
                diagnostics,
                output,
                statusBarItem
            });
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand("harperReview.clearDiagnostics", () => {
            diagnostics.clear();
            suggestionStore.clear();
            statusBarItem.text = "$(search-view-icon) Harper Review";
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand("harperReview.applyAllFixes", async () => {
            await applyAllFixes(diagnostics, output);
        })
    );

    context.subscriptions.push(
        vscode.languages.registerCodeActionsProvider(
            { scheme: "file" },
            new HarperCodeActionProvider(),
            { providedCodeActionKinds: [vscode.CodeActionKind.QuickFix] }
        )
    );

    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument(async (document) => {
            if (!getConfig().get("autoReviewOnSave")) {
                return;
            }

            const activeEditor = vscode.window.activeTextEditor;
            if (!activeEditor || activeEditor.document.uri.toString() !== document.uri.toString()) {
                return;
            }

            await reviewEditor(activeEditor, {
                selectionOnly: false,
                diagnostics,
                output,
                statusBarItem,
                showNotifications: false
            });
        })
    );
}

function deactivate() {}

async function reviewActiveEditor(options) {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showWarningMessage("No active editor to review.");
        return;
    }

    await reviewEditor(editor, {
        ...options,
        showNotifications: true
    });
}

async function reviewEditor(editor, options) {
    const document = editor.document;
    const selection = editor.selection;
    const reviewSelection = options.selectionOnly && !selection.isEmpty ? selection : null;
    const payload = {
        file_path: getFilePathForRequest(document),
        content: document.getText(),
        language: document.languageId,
        workspace_root: vscode.workspace.getWorkspaceFolder(document.uri)?.uri.fsPath,
        instructions: getConfig().get("instructions"),
        selection: reviewSelection ? toRequestRange(reviewSelection) : undefined,
        max_findings: getConfig().get("maxFindings")
    };

    statusLoading(options.statusBarItem);

    const reviewScope = reviewSelection ? "selection" : "file";
    options.output.appendLine(`[Harper Review] Requesting ${reviewScope} review for ${document.uri.fsPath}`);

    try {
        const response = await fetchReview(payload);
        applyReviewResult(document, response, options.diagnostics);

        const findingCount = Array.isArray(response.findings) ? response.findings.length : 0;
        options.statusBarItem.text = findingCount > 0
            ? `$(warning) Harper ${findingCount}`
            : "$(check) Harper Clean";

        options.output.appendLine(`[Harper Review] ${response.summary}`);
        if (options.showNotifications) {
            const message = findingCount > 0
                ? `Harper found ${findingCount} issue${findingCount === 1 ? "" : "s"} in this ${reviewScope}.`
                : `Harper found no issues in this ${reviewScope}.`;
            vscode.window.showInformationMessage(message);
        }
    } catch (error) {
        options.statusBarItem.text = "$(error) Harper Review";
        options.output.appendLine(`[Harper Review] ${error.message}`);
        vscode.window.showErrorMessage(`Harper review failed: ${error.message}`);
    }
}

function applyReviewResult(document, response, diagnosticsCollection) {
    const findings = Array.isArray(response.findings) ? response.findings : [];
    const diagnostics = [];
    const suggestions = new Map();

    findings.forEach((finding, index) => {
        const range = toVsCodeRange(document, finding.range);
        const diagnostic = new vscode.Diagnostic(
            range,
            formatDiagnosticMessage(finding),
            mapSeverity(finding.severity)
        );
        const id = `harper-review-${index}`;
        diagnostic.code = id;
        diagnostic.source = DIAGNOSTIC_SOURCE;
        diagnostics.push(diagnostic);

        if (finding.suggestion && typeof finding.suggestion.replacement === "string") {
            suggestions.set(id, {
                title: finding.title,
                description: finding.suggestion.description || "Apply Harper suggestion",
                replacement: finding.suggestion.replacement,
                range
            });
        }
    });

    diagnosticsCollection.set(document.uri, diagnostics);
    suggestionStore.set(document.uri.toString(), suggestions);
}

async function applyAllFixes(diagnosticsCollection, output) {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showWarningMessage("No active editor to fix.");
        return;
    }

    const suggestions = suggestionStore.get(editor.document.uri.toString());
    if (!suggestions || suggestions.size === 0) {
        vscode.window.showInformationMessage("No Harper fixes are available for this file.");
        return;
    }

    const entries = Array.from(suggestions.values()).sort((left, right) => {
        if (left.range.start.line !== right.range.start.line) {
            return right.range.start.line - left.range.start.line;
        }
        return right.range.start.character - left.range.start.character;
    });

    await editor.edit((editBuilder) => {
        for (const entry of entries) {
            editBuilder.replace(entry.range, entry.replacement);
        }
    });

    diagnosticsCollection.delete(editor.document.uri);
    suggestionStore.delete(editor.document.uri.toString());
    output.appendLine(`[Harper Review] Applied ${entries.length} suggestion(s) to ${editor.document.uri.fsPath}`);
    vscode.window.showInformationMessage(`Applied ${entries.length} Harper suggestion(s).`);
}

class HarperCodeActionProvider {
    provideCodeActions(document, _range, context) {
        const suggestions = suggestionStore.get(document.uri.toString());
        if (!suggestions) {
            return [];
        }

        const actions = [];

        for (const diagnostic of context.diagnostics) {
            const key = typeof diagnostic.code === "string" ? diagnostic.code : diagnostic.code?.value;
            if (!key || !suggestions.has(key)) {
                continue;
            }

            const suggestion = suggestions.get(key);
            const action = new vscode.CodeAction(
                suggestion.description || `Apply fix for ${suggestion.title}`,
                vscode.CodeActionKind.QuickFix
            );
            action.diagnostics = [diagnostic];
            action.edit = new vscode.WorkspaceEdit();
            action.edit.replace(document.uri, suggestion.range, suggestion.replacement);
            action.isPreferred = true;
            actions.push(action);
        }

        if (actions.length > 0) {
            const applyAll = new vscode.CodeAction(
                "Apply all Harper suggestions",
                vscode.CodeActionKind.QuickFix
            );
            applyAll.command = {
                command: "harperReview.applyAllFixes",
                title: "Apply all Harper suggestions"
            };
            actions.push(applyAll);
        }

        return actions;
    }
}

function toRequestRange(selection) {
    return {
        start_line: selection.start.line + 1,
        start_column: selection.start.character + 1,
        end_line: selection.end.line + 1,
        end_column: selection.end.character + 1
    };
}

function toVsCodeRange(document, range) {
    const lastLine = Math.max(0, document.lineCount - 1);
    const startLine = clamp((range?.start_line || 1) - 1, 0, lastLine);
    const endLine = clamp((range?.end_line || range?.start_line || 1) - 1, startLine, lastLine);
    const startMax = document.lineAt(startLine).text.length;
    const endMax = document.lineAt(endLine).text.length;
    const startChar = clamp((range?.start_column || 1) - 1, 0, startMax);
    const endChar = clamp((range?.end_column || range?.start_column || 1) - 1, startChar, endMax);
    return new vscode.Range(startLine, startChar, endLine, endChar);
}

function mapSeverity(severity) {
    switch ((severity || "").toLowerCase()) {
        case "error":
            return vscode.DiagnosticSeverity.Error;
        case "warning":
            return vscode.DiagnosticSeverity.Warning;
        default:
            return vscode.DiagnosticSeverity.Information;
    }
}

function formatDiagnosticMessage(finding) {
    if (finding.title && finding.message) {
        return `${finding.title}: ${finding.message}`;
    }
    return finding.message || finding.title || "Harper review finding";
}

async function fetchReview(payload) {
    const config = getConfig();
    const serverUrl = String(config.get("serverUrl") || "http://127.0.0.1:8081").replace(/\/$/, "");
    const timeoutMs = Number(config.get("requestTimeoutMs") || 30000);
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), timeoutMs);

    try {
        const response = await fetch(`${serverUrl}/api/review`, {
            method: "POST",
            headers: {
                "content-type": "application/json"
            },
            body: JSON.stringify(payload),
            signal: controller.signal
        });

        const text = await response.text();
        if (!response.ok) {
            throw new Error(text || `HTTP ${response.status}`);
        }

        return JSON.parse(text);
    } catch (error) {
        if (error.name === "AbortError") {
            throw new Error(`Request timed out after ${timeoutMs}ms`);
        }
        throw error;
    } finally {
        clearTimeout(timeout);
    }
}

function getFilePathForRequest(document) {
    const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
    if (workspaceFolder) {
        return vscode.workspace.asRelativePath(document.uri, false);
    }
    return document.uri.fsPath || document.fileName || "untitled";
}

function getConfig() {
    return vscode.workspace.getConfiguration("harperReview");
}

function statusLoading(statusBarItem) {
    statusBarItem.text = "$(sync~spin) Harper Review";
}

function clamp(value, min, max) {
    return Math.min(Math.max(value, min), max);
}

module.exports = {
    activate,
    deactivate
};
