"use strict";
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
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
// Import the Wasm core
const zq = __importStar(require("./pkg/zq_core.js"));
let decorationTypes = new Map();
let defaultTheme = null;
function hexToRgb(hex) {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? [
        parseInt(result[1], 16),
        parseInt(result[2], 16),
        parseInt(result[3], 16)
    ] : null;
}
function activate(context) {
    console.log('ZhuQian Labels extension is now active!');
    // Initialize with default theme from Rust
    try {
        defaultTheme = JSON.parse(zq.get_default_theme_json());
    }
    catch (e) {
        console.error("Failed to load default theme:", e);
    }
    let activeEditor = vscode.window.activeTextEditor;
    let statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.command = 'zhuqian.toggle';
    context.subscriptions.push(statusBarItem);
    function updateDecorations() {
        if (!activeEditor || !defaultTheme)
            return;
        const config = vscode.workspace.getConfiguration('zhuqian');
        const isEnabled = config.get('enabled', true);
        // Update Status Bar
        statusBarItem.text = isEnabled ? `$(list-unordered) 竹 (ON)` : `$(circle-slash) 竹 (OFF)`;
        statusBarItem.tooltip = isEnabled ? "ZhuQian Highlighting: Enabled" : "ZhuQian Highlighting: Disabled";
        statusBarItem.show();
        if (!isEnabled) {
            // Clear all decorations
            for (const type of decorationTypes.values()) {
                activeEditor.setDecorations(type, []);
            }
            return;
        }
        const accentHex = config.get('accentColorHl', '#b4a000');
        const textHex = config.get('textMain', '#1e1e1e');
        const customRules = config.get('highlightRules', '');
        const accent = hexToRgb(accentHex) || [180, 160, 0];
        const text_main = hexToRgb(textHex) || [30, 30, 30];
        const themeCtx = JSON.stringify({
            accent_hl: accent,
            text_main: text_main
        });
        let rulesJson = customRules.trim();
        if (!rulesJson) {
            rulesJson = JSON.stringify(defaultTheme.highlight_rules);
        }
        const text = activeEditor.document.getText();
        try {
            // Call Rust Wasm with theme context
            const spans = zq.parse_to_spans_wasm(text, rulesJson, themeCtx);
            // Map to decorations
            const decGroups = new Map();
            for (const span of spans) {
                const styleKey = JSON.stringify({
                    fg: span.fg,
                    bg: span.bg,
                    bold: span.bold,
                    italic: span.italic,
                    strike: span.strikethrough,
                    size: span.size_mult
                });
                if (!decGroups.has(styleKey))
                    decGroups.set(styleKey, []);
                const startPos = activeEditor.document.positionAt(span.start);
                const endPos = activeEditor.document.positionAt(span.end);
                const range = new vscode.Range(startPos, endPos);
                decGroups.get(styleKey).push({ range });
            }
            // Clear old and Apply new
            // First, for any existing type not in the new set, clear it
            for (const [key, type] of decorationTypes.entries()) {
                if (!decGroups.has(key)) {
                    activeEditor.setDecorations(type, []);
                }
            }
            // Apply new decorations
            for (const [key, ranges] of decGroups.entries()) {
                if (!decorationTypes.has(key)) {
                    const style = JSON.parse(key);
                    const decoType = vscode.window.createTextEditorDecorationType({
                        color: style.fg ? `rgb(${style.fg[0]},${style.fg[1]},${style.fg[2]})` : undefined,
                        backgroundColor: style.bg ? `rgba(${style.bg[0]},${style.bg[1]},${style.bg[2]}, 0.3)` : undefined,
                        fontWeight: style.bold ? 'bold' : 'normal',
                        fontStyle: style.italic ? 'italic' : 'normal',
                        textDecoration: style.strike ? 'line-through' : 'none',
                        // Note: font size multiplier is hard to map to decorations directly without changing font family/size globally
                    });
                    decorationTypes.set(key, decoType);
                }
                activeEditor.setDecorations(decorationTypes.get(key), ranges);
            }
        }
        catch (err) {
            console.error("ZhuQian Highlight Error:", err);
        }
    }
    vscode.window.onDidChangeActiveTextEditor(editor => {
        activeEditor = editor;
        if (editor)
            updateDecorations();
    }, null, context.subscriptions);
    vscode.workspace.onDidChangeTextDocument(event => {
        if (activeEditor && event.document === activeEditor.document) {
            updateDecorations();
        }
    }, null, context.subscriptions);
    vscode.workspace.onDidChangeConfiguration(e => {
        if (e.affectsConfiguration('zhuqian')) {
            updateDecorations();
        }
    }, null, context.subscriptions);
    if (activeEditor)
        updateDecorations();
    // Register Commands
    context.subscriptions.push(vscode.commands.registerCommand('zhuqian.toggle', () => {
        const config = vscode.workspace.getConfiguration('zhuqian');
        const currentState = config.get('enabled', true);
        config.update('enabled', !currentState, vscode.ConfigurationTarget.Global);
    }), vscode.commands.registerCommand('zhuqian.openSettings', () => {
        ZqSettingsPanel.createOrShow(context.extensionUri);
    }));
}
class ZqSettingsPanel {
    static currentPanel;
    _panel;
    _extensionUri;
    _disposables = [];
    constructor(panel, extensionUri) {
        this._panel = panel;
        this._extensionUri = extensionUri;
        this._update();
        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);
        this._panel.webview.onDidReceiveMessage(message => {
            switch (message.command) {
                case 'updateSetting':
                    vscode.workspace.getConfiguration('zhuqian').update(message.key, message.value, vscode.ConfigurationTarget.Global);
                    return;
                case 'openKeybindings':
                    vscode.commands.executeCommand('workbench.action.openGlobalKeybindings', 'zhuqian');
                    return;
            }
        }, null, this._disposables);
    }
    static createOrShow(extensionUri) {
        const column = vscode.window.activeTextEditor ? vscode.window.activeTextEditor.viewColumn : undefined;
        if (ZqSettingsPanel.currentPanel) {
            ZqSettingsPanel.currentPanel._panel.reveal(column);
            return;
        }
        const panel = vscode.window.createWebviewPanel('zqSettings', 'ZhuQian Settings', column || vscode.ViewColumn.One, {
            enableScripts: true,
            localResourceRoots: [extensionUri]
        });
        ZqSettingsPanel.currentPanel = new ZqSettingsPanel(panel, extensionUri);
    }
    dispose() {
        ZqSettingsPanel.currentPanel = undefined;
        this._panel.dispose();
        while (this._disposables.length) {
            const x = this._disposables.pop();
            if (x)
                x.dispose();
        }
    }
    _update() {
        this._panel.webview.html = this._getHtmlForWebview();
    }
    _getHtmlForWebview() {
        const config = vscode.workspace.getConfiguration('zhuqian');
        const settings = {
            accentColorUi: config.get('accentColorUi'),
            accentColorHl: config.get('accentColorHl'),
            bgMain: config.get('bgMain'),
            bgSide: config.get('bgSide'),
            textMain: config.get('textMain'),
            textSide: config.get('textSide'),
            fontSize: config.get('fontSize'),
            markdownRender: config.get('markdownRender'),
            highlightRules: config.get('highlightRules') || '[]',
            enabled: config.get('enabled'),
        };
        return `<!DOCTYPE html>
			<html lang="en">
			<head>
				<meta charset="UTF-8">
				<meta name="viewport" content="width=device-width, initial-scale=1.0">
				<title>ZhuQian Settings</title>
				<style>
					body { font-family: 'Segoe UI', sans-serif; background: #1e1e1e; color: #d4d4d4; padding: 20px; }
					.container { max-width: 600px; margin: 0 auto; border: 1px solid #454545; background: #252526; padding: 20px; }
					h1 { color: #8ec450; font-size: 1.5rem; margin-bottom: 20px; border-bottom: 1px solid #454545; padding-bottom: 10px; }
					.row { display: flex; align-items: center; margin-bottom: 12px; }
					.label { flex: 1; font-size: 13px; }
					.input-wrap { flex: 1; display: flex; align-items: center; }
					input[type="color"] { width: 40px; height: 24px; border: none; background: none; cursor: pointer; padding: 0; }
					input[type="text"], input[type="number"] { background: #3c3c3c; border: 1px solid #3c3c3c; color: white; padding: 2px 6px; font-family: monospace; font-size: 12px; }
					.hex-val { margin-left: 10px; font-family: monospace; font-size: 11px; color: #888; }
					.section-title { color: #8ec450; font-size: 11px; margin-top: 25px; margin-bottom: 10px; text-transform: uppercase; letter-spacing: 1px; font-weight: bold; }
					.rules-list { margin-top: 10px; }
					.rule-item { background: #333; padding: 8px; margin-bottom: 5px; border-left: 3px solid #8ec450; display: flex; align-items: center; gap: 8px; }
					.rule-item input { flex: 1; }
					.rule-item .remove { cursor: pointer; color: #f44; font-weight: bold; padding: 0 5px; }
					.add-rule { margin-top: 10px; display: flex; gap: 5px; }
					.btn-sm { background: #8ec450; color: #1e1e1e; border: none; padding: 4px 10px; cursor: pointer; font-size: 11px; font-weight: bold; }
				</style>
			</head>
			<body>
				<div class="container">
					<h1>竹签 ZhuQian (Desktop Style)</h1>
					
					<div class="section-title">Common</div>
					<div class="row">
						<div class="label">Enable Rendering</div>
						<div class="input-wrap"><input type="checkbox" id="enabled" ${settings.enabled ? 'checked' : ''} onchange="update('enabled', this.checked)"></div>
					</div>
					<div class="row">
						<div class="label">Keybinding (Default)</div>
						<div class="input-wrap">
							<span style="font-size: 11px; color: #8ec450; font-family: monospace; border: 1px solid #454545; padding: 2px 6px;">Ctrl + Alt + Z</span>
							<button class="btn-sm" style="margin-left: 10px;" onclick="openKeybindings()">REBIND</button>
						</div>
					</div>

					<div class="section-title">Color Palette</div>
					<div class="row"><div class="label">UI Accent</div><div class="input-wrap"><input type="color" id="accentColorUi" value="${settings.accentColorUi}" onchange="update('accentColorUi', this.value)"><span class="hex-val">${settings.accentColorUi}</span></div></div>
					<div class="row"><div class="label">HL Accent</div><div class="input-wrap"><input type="color" id="accentColorHl" value="${settings.accentColorHl}" onchange="update('accentColorHl', this.value)"><span class="hex-val">${settings.accentColorHl}</span></div></div>
					<div class="row"><div class="label">Editor BG</div><div class="input-wrap"><input type="color" id="bgMain" value="${settings.bgMain}" onchange="update('bgMain', this.value)"><span class="hex-val">${settings.bgMain}</span></div></div>
					<div class="row"><div class="label">Main Text</div><div class="input-wrap"><input type="color" id="textMain" value="${settings.textMain}" onchange="update('textMain', this.value)"><span class="hex-val">${settings.textMain}</span></div></div>

					<div class="section-title">Highlight Rules</div>
					<div id="rulesList" class="rules-list"></div>
					<div class="add-rule">
						<input type="text" id="newRuleName" placeholder="Name" style="width: 80px;">
						<input type="text" id="newRulePattern" placeholder="Regex Pattern" style="flex: 1;">
						<input type="color" id="newRuleColor" value="#ff4444">
						<button class="btn-sm" onclick="addRule()">ADD</button>
					</div>

					<div class="section-title">Display Options</div>
					<div class="row">
						<div class="label">Font Size (px)</div>
						<div class="input-wrap"><input type="number" id="fontSize" value="${settings.fontSize}" onchange="update('fontSize', parseInt(this.value))" style="width: 50px;"></div>
					</div>
					<div class="row">
						<div class="label">Markdown Enhancement</div>
						<div class="input-wrap"><input type="checkbox" id="markdownRender" ${settings.markdownRender ? 'checked' : ''} onchange="update('markdownRender', this.checked)"></div>
					</div>
				</div>

				<script>
					const vscode = acquireVsCodeApi();
					let rules = [];
					try { rules = JSON.parse(\`${settings.highlightRules}\`); } catch(e) { rules = []; }

					function renderRules() {
						const list = document.getElementById('rulesList');
						list.innerHTML = '';
						rules.forEach((r, i) => {
							const div = document.createElement('div');
							div.className = 'rule-item';
							div.innerHTML = \`
								<span style="font-size: 11px; width: 60px; overflow: hidden; text-overflow: ellipsis;">\${r.name}</span>
								<input type="text" value="\${r.pattern}" onchange="updateRule(\${i}, 'pattern', this.value)">
								<input type="color" value="\${rgbToHex(r.color)}" onchange="updateRule(\${i}, 'color', hexToRgb(this.value))">
								<span class="remove" onclick="removeRule(\${i})">×</span>
							\`;
							list.appendChild(div);
						});
					}

					function update(key, value) {
						vscode.postMessage({ command: 'updateSetting', key, value });
						if (key.includes('Color')) {
							const row = document.getElementById(key).parentElement;
							const label = row.querySelector('.hex-val');
							if (label) label.textContent = value.toUpperCase();
						}
					}

					function openKeybindings() {
						vscode.postMessage({ command: 'openKeybindings' });
					}

					function updateRule(i, key, val) {
						rules[i][key] = val;
						update('highlightRules', JSON.stringify(rules));
					}

					function addRule() {
						const name = document.getElementById('newRuleName').value;
						const pattern = document.getElementById('newRulePattern').value;
						const color = hexToRgb(document.getElementById('newRuleColor').value);
						if (!name || !pattern) return;
						rules.push({ name, pattern, color, bold: false, is_background: true });
						update('highlightRules', JSON.stringify(rules));
						renderRules();
						document.getElementById('newRuleName').value = '';
						document.getElementById('newRulePattern').value = '';
					}

					function removeRule(i) {
						rules.splice(i, 1);
						update('highlightRules', JSON.stringify(rules));
						renderRules();
					}

					function hexToRgb(hex) {
						const r = parseInt(hex.slice(1, 3), 16);
						const g = parseInt(hex.slice(3, 5), 16);
						const b = parseInt(hex.slice(5, 7), 16);
						return [r, g, b];
					}

					function rgbToHex(rgb) {
						return "#" + rgb.map(x => x.toString(16).padStart(2, '0')).join('');
					}

					renderRules();
				</script>
			</body>
			</html>`;
    }
}
function deactivate() {
    for (const type of decorationTypes.values()) {
        type.dispose();
    }
}
//# sourceMappingURL=extension.js.map