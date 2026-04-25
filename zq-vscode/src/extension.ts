import * as vscode from 'vscode';
// Import the Wasm core (bundler target: functions exported from zq_core_bg.js)
import * as zq from './pkg/zq_core.js';

let decorationTypes = new Map<string, vscode.TextEditorDecorationType>();
let defaultTheme: any = null;

function hexToRgb(hex: string): [number, number, number] | null {
	const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
	return result ? [
		parseInt(result[1], 16),
		parseInt(result[2], 16),
		parseInt(result[3], 16)
	] : null;
}

export function activate(context: vscode.ExtensionContext) {
	console.log('ZhuQian Labels extension is now active!');

	// Initialize with default theme from Rust
	try {
		defaultTheme = JSON.parse(zq.get_default_theme_json());
	} catch (e) {
		console.error("Failed to load default theme:", e);
	}

	let activeEditor = vscode.window.activeTextEditor;
	let statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
	statusBarItem.command = 'zhuqian.toggle';
	context.subscriptions.push(statusBarItem);

	let statsBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 10);
	context.subscriptions.push(statsBarItem);

	function updateStats(text: string) {
		const wordCount = (text.match(/[\u4e00-\u9fa5]|\w+/g) || []).length;
		const readingTime = Math.ceil(wordCount / 300); // ~300 words per minute
		statsBarItem.text = `$(note) ${wordCount} 字 | $(clock) ${readingTime} 分钟`;
		statsBarItem.show();
	}

	function updateDecorations() {
		if (!activeEditor || !defaultTheme) return;

		const text = activeEditor.document.getText();
		updateStats(text);

		const config = vscode.workspace.getConfiguration('zhuqian');
		const isEnabled = config.get<boolean>('enabled', true);

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

		const accentHex = config.get<string>('accentColorHl', '#b4a000');
		const textHex = config.get<string>('textMain', '#1e1e1e');
		const customRules = config.get<string>('highlightRules', '');

		const accent = hexToRgb(accentHex) || [180, 160, 0];
		const text_main = hexToRgb(textHex) || [30, 30, 30];

		// Build level colors from settings (6 levels)
		const defaultLevelColors = [
			[230, 80, 80], [80, 200, 80], [80, 80, 230],
			[230, 180, 50], [180, 80, 200], [80, 200, 200]
		];
		const levelColorsRaw = config.get<number[][]>('levelColors', defaultLevelColors);
		const level_colors = levelColorsRaw.slice(0, 6).map(c =>
			Array.isArray(c) && c.length >= 3 ? c.slice(0, 3) : [150, 150, 150]
		);

		const themeCtx = JSON.stringify({
			accent_hl: accent,
			text_main: text_main,
			hide_labels: config.get<boolean>('hideLabels', false),
			level_colors: level_colors
		});

		try {
			// Call Rust Wasm with theme context
			const spans: any[] = zq.parse_to_spans_wasm(text, themeCtx);

			// Map to decorations
			const decGroups = new Map<string, vscode.DecorationOptions[]>();

			for (const span of spans) {
				const styleKey = JSON.stringify({
					fg: span.fg,
					bg: span.bg,
					bold: span.bold,
					italic: span.italic,
					strike: span.strikethrough,
					size: span.size_mult,
					hidden: span.is_hidden
				});

				if (!decGroups.has(styleKey)) decGroups.set(styleKey, []);

				const startPos = activeEditor.document.positionAt(span.start);
				const endPos = activeEditor.document.positionAt(span.end);
				const range = new vscode.Range(startPos, endPos);

				decGroups.get(styleKey)!.push({ range });
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
						color: style.hidden ? 'transparent' : (style.fg ? `rgb(${style.fg[0]},${style.fg[1]},${style.fg[2]})` : undefined),
						backgroundColor: style.bg ? `rgba(${style.bg[0]},${style.bg[1]},${style.bg[2]}, 0.3)` : undefined,
						fontWeight: style.bold ? 'bold' : 'normal',
						fontStyle: style.italic ? 'italic' : 'normal',
						textDecoration: style.strike ? 'line-through' : (style.hidden ? 'none; display: none;' : 'none'),
						opacity: style.hidden ? '0' : '1',
						letterSpacing: style.hidden ? '-100ch' : 'normal'
					});
					decorationTypes.set(key, decoType);
				}
				activeEditor.setDecorations(decorationTypes.get(key)!, ranges);
			}

		} catch (err) {
			console.error("ZhuQian Highlight Error:", err);
		}
	}

	vscode.window.onDidChangeActiveTextEditor(editor => {
		activeEditor = editor;
		if (editor) updateDecorations();
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

	context.subscriptions.push(
		vscode.languages.registerDocumentSymbolProvider(
			{ language: 'zq' },
			{
				provideDocumentSymbols(document, token) {
					const symbols: vscode.DocumentSymbol[] = [];
					const text = document.getText();

					// 1. Headings (Markdown style)
					const headings = lineByLineHeadings(text);
					symbols.push(...headings);

					// 2. Semantic Labels (Extracted via Wasm)
					try {
						const labels: any[] = zq.parse_semantic_labels_wasm(text);

						// Build tree
						const rootSymbols: vscode.DocumentSymbol[] = [];
						const stack: { depth: number, symbol: vscode.DocumentSymbol }[] = [];

						for (const label of labels) {
							const pos = document.positionAt(label.start_byte);
							const range = new vscode.Range(pos.line, 0, pos.line, 100);

							let kind = vscode.SymbolKind.Key;
							if (label.depth === 1) kind = vscode.SymbolKind.Class;
							else if (label.depth === 2) kind = vscode.SymbolKind.Interface;

							const detail = label.properties?.length > 0 ? `| ${label.properties.join(' | ')}` : '';
							let nameStr = label.category || '[]';
							if (label.text) {
								nameStr += ` - ${label.text}`;
							}

							const symbol = new vscode.DocumentSymbol(nameStr, detail, kind, range, range);

							// Nesting logic
							while (stack.length > 0 && label.depth <= stack[stack.length - 1].depth) {
								stack.pop();
							}

							if (stack.length === 0) {
								rootSymbols.push(symbol);
							} else {
								stack[stack.length - 1].symbol.children.push(symbol);
							}
							stack.push({ depth: label.depth, symbol });
						}
						symbols.push(...rootSymbols);
					} catch (e) {
						console.error("Failed to extract semantic labels:", e);
					}

					return symbols;
				}
			}
		)
	);

	function lineByLineHeadings(text: string): vscode.DocumentSymbol[] {
		const syms: vscode.DocumentSymbol[] = [];
		const lines = text.split('\n');
		lines.forEach((line, i) => {
			const headingMatch = line.match(/^(#+)\s+(.+)$/);
			if (headingMatch) {
				const level = headingMatch[1].length;
				const name = headingMatch[2];
				syms.push(new vscode.DocumentSymbol(
					name, `Level ${level}`,
					vscode.SymbolKind.Module,
					new vscode.Range(i, 0, i, line.length),
					new vscode.Range(i, 0, i, line.length)
				));
			}
		});
		return syms;
	}

	// Register Commands
	context.subscriptions.push(
		vscode.commands.registerCommand('zhuqian.toggle', () => {
			const config = vscode.workspace.getConfiguration('zhuqian');
			const currentState = config.get<boolean>('enabled', true);
			config.update('enabled', !currentState, vscode.ConfigurationTarget.Global);
		}),
		vscode.commands.registerCommand('zhuqian.openSettings', () => {
			ZqSettingsPanel.createOrShow(context.extensionUri);
		}),
		vscode.commands.registerCommand('zhuqian.copyCleanText', () => {
			if (!activeEditor) return;
			const text = activeEditor.document.getText();
			const clean = zq.strip_semantic_labels_wasm(text);
			vscode.env.clipboard.writeText(clean);
			vscode.window.showInformationMessage("Clean text copied to clipboard.");
		}),
		vscode.commands.registerCommand('zhuqian.toggleHideLabels', () => {
			const config = vscode.workspace.getConfiguration('zhuqian');
			const current = config.get<boolean>('hideLabels', false);
			config.update('hideLabels', !current, vscode.ConfigurationTarget.Global);
			vscode.window.setStatusBarMessage(
				current ? '竹签: Labels visible' : '竹签: Labels hidden',
				2000
			);
		})
	);

	// Register sidebar provider
	const sidebarProvider = new ZqSidebarProvider(context.extensionUri);
	context.subscriptions.push(
		vscode.window.registerWebviewViewProvider('zhuqian.sidebarView', sidebarProvider)
	);

	// Refresh sidebar when settings change
	context.subscriptions.push(
		vscode.workspace.onDidChangeConfiguration(e => {
			if (e.affectsConfiguration('zhuqian')) {
				sidebarProvider.refresh();
			}
		})
	);
}

class ZqSidebarProvider implements vscode.WebviewViewProvider {
	private _view?: vscode.WebviewView;
	constructor(private readonly _extensionUri: vscode.Uri) {}
	public resolveWebviewView(webviewView: vscode.WebviewView) {
		this._view = webviewView;
		webviewView.webview.options = { enableScripts: true };
		this.refresh();
	}
	public refresh() {
		if (this._view) {
			this._view.webview.html = `<html><body>Sidebar Content</body></html>`;
		}
	}
}

class ZqSettingsPanel {
	public static currentPanel: ZqSettingsPanel | undefined;
	private readonly _panel: vscode.WebviewPanel;
	private readonly _extensionUri: vscode.Uri;
	private _disposables: vscode.Disposable[] = [];

	private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {
		this._panel = panel;
		this._extensionUri = extensionUri;
		this._update();
		this._panel.onDidDispose(() => this.dispose(), null, this._disposables);
		this._panel.webview.onDidReceiveMessage(
			message => {
				switch (message.command) {
					case 'updateSetting':
						vscode.workspace.getConfiguration('zhuqian').update(message.key, message.value, vscode.ConfigurationTarget.Global);

						// Sync to [zq] language settings for font/size
						if (message.key === 'fontFamily' || message.key === 'fontSize') {
							const zqConfig = vscode.workspace.getConfiguration('[zq]');
							if (message.key === 'fontFamily') {
								zqConfig.update('editor.fontFamily', message.value, vscode.ConfigurationTarget.Global);
							} else if (message.key === 'fontSize') {
								zqConfig.update('editor.fontSize', message.value, vscode.ConfigurationTarget.Global);
							}
						}
						return;
					case 'applyTheme':
						const theme = message.theme;
						const c = vscode.workspace.getConfiguration('zhuqian');
						if (theme === 'light') {
							c.update('bgMain', '#f8f6eb', true);
							c.update('accentColorUi', '#8eac50', true);
							c.update('accentColorHl', '#b4a000', true);
							c.update('textMain', '#1e1e1e', true);
						} else if (theme === 'dark') {
							c.update('bgMain', '#1e1e1e', true);
							c.update('accentColorUi', '#a0c83c', true);
							c.update('accentColorHl', '#ffe600', true);
							c.update('textMain', '#dcdcdc', true);
						}
						return;
					case 'openKeybindings':
						vscode.commands.executeCommand('workbench.action.openGlobalKeybindings', 'zhuqian');
						return;
				}
			},
			null,
			this._disposables
		);
	}

	public static createOrShow(extensionUri: vscode.Uri) {
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

	public dispose() {
		ZqSettingsPanel.currentPanel = undefined;
		this._panel.dispose();
		while (this._disposables.length) {
			const x = this._disposables.pop();
			if (x) x.dispose();
		}
	}

	private _update() {
		this._panel.webview.html = this._getHtmlForWebview();
	}

	private _getHtmlForWebview() {
		const config = vscode.workspace.getConfiguration('zhuqian');
		const ff = config.get<string>('fontFamily') || 'serif';
		const settings = {
			accentColorUi: config.get('accentColorUi'),
			accentColorHl: config.get('accentColorHl'),
			bgMain: config.get('bgMain'),
			bgSide: config.get('bgSide'),
			textMain: config.get('textMain'),
			textSide: config.get('textSide'),
			fontSize: config.get('fontSize'),
			fontFamily: ff,
			markdownRender: config.get('markdownRender'),
			highlightRules: config.get('highlightRules') || '[]',
			enabled: config.get('enabled'),
			hideLabels: config.get('hideLabels'),
			levelColors: JSON.stringify(config.get('levelColors') || [
				[230,80,80],[80,200,80],[80,80,230],[230,180,50],[180,80,200],[80,200,200]
			]),
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
					input[type="text"], input[type="number"], select { background: #3c3c3c; border: 1px solid #3c3c3c; color: white; padding: 2px 6px; font-family: monospace; font-size: 12px; }
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
						<div class="label">Keybinding</div>
						<div class="input-wrap">
							<span style="font-size: 11px; color: #8ec450; font-family: monospace; border: 1px solid #454545; padding: 2px 6px;">Ctrl + Alt + Z</span>
							<button class="btn-sm" style="margin-left: 10px;" onclick="openKeybindings()">REBIND</button>
						</div>
					</div>

					<div class="section-title">Themes (Presets)</div>
					<div class="row" style="gap: 10px;">
						<button class="btn-sm" style="flex: 1; height: 30px;" onclick="applyTheme('light')">Default (Light)</button>
						<button class="btn-sm" style="flex: 1; height: 30px; background: #333; color: #a0c83c; border: 1px solid #454545;" onclick="applyTheme('dark')">Default (Dark)</button>
					</div>

					<div class="section-title">Font Settings (Exclusive to .zq)</div>
					<div class="row">
						<div class="label">Editor Font Family</div>
						<div class="input-wrap">
							<select id="fontFamily" onchange="update('fontFamily', this.value)">
								<option value="serif" ${settings.fontFamily === 'serif' ? 'selected' : ''}>Serif (SongTi)</option>
								<option value="sans-serif" ${settings.fontFamily === 'sans-serif' ? 'selected' : ''}>Sans-Serif (YaHei)</option>
								<option value="monospace" ${settings.fontFamily === 'monospace' ? 'selected' : ''}>Monospace</option>
								<option value="'Kaiti', 'STKaiti', serif" ${settings.fontFamily.indexOf('Kaiti') !== -1 ? 'selected' : ''}>KaiTi</option>
							</select>
							<input type="text" placeholder="Custom Font" onchange="update('fontFamily', this.value)" style="margin-left: 5px; width: 80px;">
						</div>
					</div>
					<div class="row">
						<div class="label">Base Font Size (px)</div>
						<div class="input-wrap"><input type="number" id="fontSize" value="${settings.fontSize}" onchange="update('fontSize', parseInt(this.value))" style="width: 50px;"></div>
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
						<div class="label">Markdown Enhancement</div>
						<div class="input-wrap"><input type="checkbox" id="markdownRender" ${settings.markdownRender ? 'checked' : ''} onchange="update('markdownRender', this.checked)"></div>
					</div>
					<div class="row">
						<div class="label">Hide Labels <span style="font-size:10px;color:#888">(Ctrl+H)</span></div>
						<div class="input-wrap"><input type="checkbox" id="hideLabels" ${settings.hideLabels ? 'checked' : ''} onchange="update('hideLabels', this.checked)"></div>
					</div>

					<div class="section-title">Semantic Level Colors</div>
					<div style="font-size:11px;color:#888;margin-bottom:8px">Colors for label hierarchy: <code>[category-text]</code> = level 1, <code>[.category-text]</code> = level 2, etc.</div>
					<div id="levelColorRows"></div>
				</div>

				<script>
					const vscode = acquireVsCodeApi();
					let rules = [];
					const rulesStr = \`${settings.highlightRules}\`;
					try { rules = JSON.parse(rulesStr); } catch(e) { rules = []; }

					// Level colors
					let levelColors = [];
					try { levelColors = JSON.parse(\`${settings.levelColors}\`); } catch(e) {}
					const levelNames = ['Level 1 (depth 0)', 'Level 2 (depth .)', 'Level 3 (depth ..)', 'Level 4', 'Level 5', 'Level 6'];

					function renderLevelColors() {
						const container = document.getElementById('levelColorRows');
						container.innerHTML = '';
						levelColors.forEach((rgb, i) => {
							const hex = '#' + rgb.map(x => x.toString(16).padStart(2,'0')).join('');
							const row = document.createElement('div');
							row.className = 'row';
							row.innerHTML = \`
								<div class="label" style="display:flex;align-items:center;gap:6px">
									<span style="background:rgb(\${rgb[0]},\${rgb[1]},\${rgb[2]});width:12px;height:12px;display:inline-block;border-radius:2px"></span>
									\${levelNames[i]}
								</div>
								<div class="input-wrap">
									<input type="color" value="\${hex}" onchange="updateLevelColor(\${i}, this.value)">
									<span class="hex-val">\${hex.toUpperCase()}</span>
								</div>\`;
							container.appendChild(row);
						});
					}

					function updateLevelColor(index, hex) {
						const r = parseInt(hex.slice(1,3),16);
						const g = parseInt(hex.slice(3,5),16);
						const b = parseInt(hex.slice(5,7),16);
						levelColors[index] = [r, g, b];
						update('levelColors', levelColors);
						renderLevelColors();
					}

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

					function applyTheme(theme) {
						vscode.postMessage({ command: 'applyTheme', theme: theme });
						// Note: VS Code settings update is async, so a slight delay before refresh might be better
						setTimeout(() => location.reload(), 100);
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
						rules.push({ name, pattern, color, bold: false, is_background: true, priority: 1 });
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
					renderLevelColors();
				</script>
			</body>
			</html>`;
	}
}

export function deactivate() {
	for (const type of decorationTypes.values()) {
		type.dispose();
	}
}



