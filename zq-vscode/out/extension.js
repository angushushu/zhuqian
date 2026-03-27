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
    function updateDecorations() {
        if (!activeEditor || !defaultTheme)
            return;
        const config = vscode.workspace.getConfiguration('zhuqian');
        const accentHex = config.get('accentColor', '#ae7157');
        const customRules = config.get('highlightRules', '');
        let accent = hexToRgb(accentHex) || [174, 113, 87];
        let rulesJson = customRules.trim();
        if (!rulesJson) {
            rulesJson = JSON.stringify(defaultTheme.highlight_rules);
        }
        const text = activeEditor.document.getText();
        try {
            // Call Rust Wasm
            const spans = zq.parse_to_spans_wasm(text, rulesJson, accent[0], accent[1], accent[2]);
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
}
function deactivate() {
    for (const type of decorationTypes.values()) {
        type.dispose();
    }
}
//# sourceMappingURL=extension.js.map