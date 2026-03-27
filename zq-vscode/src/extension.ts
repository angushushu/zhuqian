import * as vscode from 'vscode';
// Import the Wasm core
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

	function updateDecorations() {
		if (!activeEditor || !defaultTheme) return;
		
		const config = vscode.workspace.getConfiguration('zhuqian');
		const accentHex = config.get<string>('accentColorHl', '#b4a000');
		const customRules = config.get<string>('highlightRules', '');

		let accent = hexToRgb(accentHex) || [180, 160, 0];
		let rulesJson = customRules.trim();
		if (!rulesJson) {
			rulesJson = JSON.stringify(defaultTheme.highlight_rules);
		}

		const text = activeEditor.document.getText();

		try {
			// Call Rust Wasm
			const spans: any[] = zq.parse_to_spans_wasm(text, rulesJson, accent[0], accent[1], accent[2]);
			
			// Map to decorations
			const decGroups = new Map<string, vscode.DecorationOptions[]>();

			for (const span of spans) {
				const styleKey = JSON.stringify({
					fg: span.fg,
					bg: span.bg,
					bold: span.bold,
					italic: span.italic,
					strike: span.strikethrough,
					size: span.size_mult
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
						color: style.fg ? `rgb(${style.fg[0]},${style.fg[1]},${style.fg[2]})` : undefined,
						backgroundColor: style.bg ? `rgba(${style.bg[0]},${style.bg[1]},${style.bg[2]}, 0.3)` : undefined,
						fontWeight: style.bold ? 'bold' : 'normal',
						fontStyle: style.italic ? 'italic' : 'normal',
						textDecoration: style.strike ? 'line-through' : 'none',
						// Note: font size multiplier is hard to map to decorations directly without changing font family/size globally
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

	if (activeEditor) updateDecorations();
}

export function deactivate() {
	for (const type of decorationTypes.values()) {
		type.dispose();
	}
}
