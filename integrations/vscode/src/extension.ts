/**
 * CodeTruth VS Code Extension
 * 
 * Provides AI Code Intelligence & Drift Monitoring directly in VS Code.
 */

import * as vscode from 'vscode';
import * as cp from 'child_process';
import * as path from 'path';

interface CTPAnalysis {
  ctp_version: string;
  explanation_id: string;
  module: {
    name: string;
    path: string;
    language: string;
    lines_of_code: number;
    complexity_score: number;
  };
  intent: {
    declared_intent: string;
    inferred_intent: string;
    confidence: number;
  };
  behavior: {
    actual_behavior: string;
    side_effects: Array<{ effect_type: string; description: string; risk_level: string }>;
  };
  drift: {
    drift_detected: boolean;
    drift_severity: string;
    drift_details: Array<{
      drift_type: string;
      expected: string;
      actual: string;
      location: { file: string; line_start: number; line_end: number };
      remediation: string;
    }>;
  };
  policies: {
    policy_results: Array<{
      policy_id: string;
      policy_name: string;
      status: string;
      violations: Array<{
        rule: string;
        severity: string;
        message: string;
        location: { file: string; line_start: number; line_end: number };
      }>;
    }>;
  };
}

let diagnosticCollection: vscode.DiagnosticCollection;
let outputChannel: vscode.OutputChannel;
let statusBarItem: vscode.StatusBarItem;

export function activate(context: vscode.ExtensionContext) {
  console.log('CodeTruth extension activated');

  // Create output channel for logs
  outputChannel = vscode.window.createOutputChannel('CodeTruth');
  context.subscriptions.push(outputChannel);

  // Create status bar item
  statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
  statusBarItem.command = 'codetruth.analyzeFile';
  statusBarItem.text = '$(shield) CTP';
  statusBarItem.tooltip = 'CodeTruth: Click to analyze current file';
  statusBarItem.show();
  context.subscriptions.push(statusBarItem);

  // Create diagnostics collection
  diagnosticCollection = vscode.languages.createDiagnosticCollection('codetruth');
  context.subscriptions.push(diagnosticCollection);

  // Register commands
  const analyzeFile = vscode.commands.registerCommand('codetruth.analyzeFile', async () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showWarningMessage('No file open');
      return;
    }

    await analyzeDocument(editor.document);
  });

  const analyzeWorkspace = vscode.commands.registerCommand('codetruth.analyzeWorkspace', async () => {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders) {
      vscode.window.showWarningMessage('No workspace open');
      return;
    }

    await vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title: 'Analyzing workspace...',
        cancellable: true,
      },
      async (progress: vscode.Progress<{ message?: string; increment?: number }>, token: vscode.CancellationToken) => {
        const files = await vscode.workspace.findFiles(
          '**/*.{py,js,ts,tsx,rs,go,java}',
          '**/node_modules/**'
        );

        let analyzed = 0;
        for (const file of files) {
          if (token.isCancellationRequested) break;

          progress.report({
            message: path.basename(file.fsPath),
            increment: 100 / files.length,
          });

          try {
            const document = await vscode.workspace.openTextDocument(file);
            await analyzeDocument(document, false);
            analyzed++;
          } catch (e) {
            outputChannel.appendLine(`Error analyzing ${file.fsPath}: ${e}`);
          }
        }

        vscode.window.showInformationMessage(`Analyzed ${analyzed} files`);
      }
    );
  });

  const checkPolicies = vscode.commands.registerCommand('codetruth.checkPolicies', async () => {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders) {
      vscode.window.showWarningMessage('No workspace open');
      return;
    }

    const cwd = workspaceFolders[0].uri.fsPath;
    
    await vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title: 'Checking policies...',
        cancellable: false,
      },
      async () => {
        try {
          const result = await runCTPCommand(['check', '--policies', '.ctp/policies/', '.'], cwd);
          outputChannel.appendLine(result);
          outputChannel.show();
          vscode.window.showInformationMessage('Policy check complete. See output for details.');
        } catch (error) {
          vscode.window.showErrorMessage(`Policy check failed: ${error}`);
        }
      }
    );
  });

  const showExplanation = vscode.commands.registerCommand('codetruth.showExplanation', async () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showWarningMessage('No file open');
      return;
    }

    const analysis = await getAnalysis(editor.document);
    if (!analysis) return;

    // Create webview panel to show explanation
    const panel = vscode.window.createWebviewPanel(
      'ctpExplanation',
      `CTP: ${path.basename(editor.document.fileName)}`,
      vscode.ViewColumn.Beside,
      { enableScripts: true }
    );

    panel.webview.html = getExplanationHtml(analysis);
  });

  context.subscriptions.push(analyzeFile, analyzeWorkspace, checkPolicies, showExplanation);

  // Watch for file saves if enabled
  const config = vscode.workspace.getConfiguration('codetruth');
  if (config.get('analyzeOnSave')) {
    context.subscriptions.push(
      vscode.workspace.onDidSaveTextDocument(async (document: vscode.TextDocument) => {
        if (isSupportedLanguage(document.languageId)) {
          await analyzeDocument(document, false);
        }
      })
    );
  }

  // Update status bar on active editor change
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor: vscode.TextEditor | undefined) => {
      if (editor && isSupportedLanguage(editor.document.languageId)) {
        statusBarItem.show();
      } else {
        statusBarItem.hide();
      }
    })
  );
}

async function analyzeDocument(document: vscode.TextDocument, showNotification = true): Promise<CTPAnalysis | undefined> {
  const analysis = await getAnalysis(document);
  if (!analysis) return undefined;

  // Update diagnostics
  updateDiagnostics(document, analysis);

  // Update status bar
  updateStatusBar(analysis);

  if (showNotification) {
    const severity = analysis.drift.drift_severity;
    const message = `${document.fileName}: ${severity} drift, ${analysis.intent.confidence * 100}% confidence`;
    
    if (severity === 'HIGH' || severity === 'CRITICAL') {
      vscode.window.showWarningMessage(message);
    } else {
      vscode.window.showInformationMessage(message);
    }
  }

  return analysis;
}

async function getAnalysis(document: vscode.TextDocument): Promise<CTPAnalysis | undefined> {
  const workspaceFolders = vscode.workspace.workspaceFolders;
  const cwd = workspaceFolders?.[0]?.uri.fsPath || path.dirname(document.fileName);

  try {
    const result = await runCTPCommand(['explain', document.fileName, '--format', 'json'], cwd);
    return JSON.parse(result) as CTPAnalysis;
  } catch (error) {
    outputChannel.appendLine(`Analysis error: ${error}`);
    vscode.window.showErrorMessage(`Analysis failed: ${error}`);
    return undefined;
  }
}

function runCTPCommand(args: string[], cwd: string): Promise<string> {
  return new Promise((resolve, reject) => {
    cp.exec(`ctp ${args.join(' ')}`, { cwd }, (error: Error | null, stdout: string, stderr: string) => {
      if (error) {
        reject(stderr || error.message);
      } else {
        resolve(stdout);
      }
    });
  });
}

function updateDiagnostics(document: vscode.TextDocument, analysis: CTPAnalysis) {
  const diagnostics: vscode.Diagnostic[] = [];

  // Add drift diagnostics
  for (const detail of analysis.drift.drift_details) {
    const range = new vscode.Range(
      Math.max(0, detail.location.line_start - 1),
      0,
      Math.max(0, detail.location.line_end - 1),
      Number.MAX_VALUE
    );

    const severity = analysis.drift.drift_severity === 'CRITICAL' || analysis.drift.drift_severity === 'HIGH'
      ? vscode.DiagnosticSeverity.Warning
      : vscode.DiagnosticSeverity.Information;

    const diagnostic = new vscode.Diagnostic(
      range,
      `[CTP Drift] ${detail.drift_type}: ${detail.remediation}`,
      severity
    );
    diagnostic.source = 'CodeTruth';
    diagnostics.push(diagnostic);
  }

  // Add policy violation diagnostics
  for (const result of analysis.policies.policy_results) {
    for (const violation of result.violations) {
      const range = new vscode.Range(
        Math.max(0, violation.location.line_start - 1),
        0,
        Math.max(0, violation.location.line_start - 1),
        Number.MAX_VALUE
      );

      const severity = violation.severity === 'CRITICAL' || violation.severity === 'ERROR'
        ? vscode.DiagnosticSeverity.Error
        : violation.severity === 'WARNING'
        ? vscode.DiagnosticSeverity.Warning
        : vscode.DiagnosticSeverity.Information;

      const diagnostic = new vscode.Diagnostic(
        range,
        `[${result.policy_name}] ${violation.message}`,
        severity
      );
      diagnostic.source = 'CodeTruth';
      diagnostics.push(diagnostic);
    }
  }

  diagnosticCollection.set(document.uri, diagnostics);
}

function updateStatusBar(analysis: CTPAnalysis) {
  const severity = analysis.drift.drift_severity;
  const icon = severity === 'NONE' ? '$(check)' :
               severity === 'LOW' ? '$(info)' :
               severity === 'MEDIUM' ? '$(warning)' : '$(error)';
  
  statusBarItem.text = `${icon} CTP: ${severity}`;
  statusBarItem.tooltip = `Drift: ${severity}\nConfidence: ${(analysis.intent.confidence * 100).toFixed(0)}%\nBehavior: ${analysis.behavior.actual_behavior}`;
}

function isSupportedLanguage(languageId: string): boolean {
  return ['python', 'javascript', 'typescript', 'typescriptreact', 'rust', 'go', 'java'].includes(languageId);
}

function getExplanationHtml(analysis: CTPAnalysis): string {
  const driftColor = analysis.drift.drift_severity === 'NONE' ? '#10b981' :
                     analysis.drift.drift_severity === 'LOW' ? '#f59e0b' :
                     analysis.drift.drift_severity === 'MEDIUM' ? '#f97316' : '#ef4444';

  return `<!DOCTYPE html>
<html>
<head>
  <style>
    body { font-family: var(--vscode-font-family); padding: 20px; color: var(--vscode-foreground); }
    h1 { color: var(--vscode-textLink-foreground); }
    h2 { border-bottom: 1px solid var(--vscode-panel-border); padding-bottom: 8px; margin-top: 24px; }
    .metric { display: inline-block; margin-right: 24px; margin-bottom: 16px; }
    .metric-value { font-size: 1.5em; font-weight: bold; }
    .metric-label { color: var(--vscode-descriptionForeground); font-size: 0.9em; }
    .drift-badge { background: ${driftColor}; color: white; padding: 4px 12px; border-radius: 4px; font-weight: bold; }
    .section { background: var(--vscode-editor-background); padding: 16px; border-radius: 8px; margin: 12px 0; }
    .side-effect { padding: 8px; margin: 4px 0; background: var(--vscode-inputValidation-warningBackground); border-radius: 4px; }
    pre { background: var(--vscode-textCodeBlock-background); padding: 12px; border-radius: 4px; overflow-x: auto; }
  </style>
</head>
<body>
  <h1>🔍 ${analysis.module.name}</h1>
  
  <div class="metric">
    <div class="metric-value">${analysis.module.lines_of_code}</div>
    <div class="metric-label">Lines of Code</div>
  </div>
  <div class="metric">
    <div class="metric-value">${analysis.module.complexity_score.toFixed(1)}</div>
    <div class="metric-label">Complexity</div>
  </div>
  <div class="metric">
    <div class="metric-value">${(analysis.intent.confidence * 100).toFixed(0)}%</div>
    <div class="metric-label">Confidence</div>
  </div>
  <div class="metric">
    <div class="drift-badge">${analysis.drift.drift_severity}</div>
    <div class="metric-label">Drift Level</div>
  </div>

  <h2>Intent</h2>
  <div class="section">
    <strong>Declared:</strong> ${analysis.intent.declared_intent || '<em>None</em>'}<br><br>
    <strong>Inferred:</strong> ${analysis.intent.inferred_intent}
  </div>

  <h2>Behavior</h2>
  <div class="section">
    <p>${analysis.behavior.actual_behavior}</p>
    ${analysis.behavior.side_effects.length > 0 ? `
      <strong>Side Effects:</strong>
      ${analysis.behavior.side_effects.map(se => `
        <div class="side-effect">
          <strong>${se.effect_type}</strong> (${se.risk_level}): ${se.description}
        </div>
      `).join('')}
    ` : ''}
  </div>

  ${analysis.drift.drift_details.length > 0 ? `
    <h2>Drift Details</h2>
    ${analysis.drift.drift_details.map(d => `
      <div class="section">
        <strong>${d.drift_type}</strong><br>
        Expected: ${d.expected}<br>
        Actual: ${d.actual}<br>
        <em>Remediation: ${d.remediation}</em>
      </div>
    `).join('')}
  ` : ''}

  <h2>Raw Analysis</h2>
  <pre>${JSON.stringify(analysis, null, 2)}</pre>
</body>
</html>`;
}

export function deactivate() {
  console.log('CodeTruth extension deactivated');
  diagnosticCollection?.dispose();
  outputChannel?.dispose();
  statusBarItem?.dispose();
}
