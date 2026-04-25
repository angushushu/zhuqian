# ZhuQian UX Master Plan - Semantic Scaffolding for Writing

## Design Philosophy

ZhuQian's core identity: **a cognitive tool that makes structural thinking visible during writing.**

Every feature must answer: *"Does this help writers build, see, or navigate their semantic scaffolding?"*

Five design principles:
1. **Calm Interface** - Writing requires focus. UI recedes when not needed.
2. **Progressive Disclosure** - Simple by default, powerful on demand.
3. **Semantic First** - Every element reinforces the scaffolding metaphor.
4. **Dual Mode** - Writing mode (minimal, focused) vs Analysis mode (rich, informative).
5. **Spatial Memory** - Visual consistency so users navigate without thinking.

---

## Phase 1: Quick Wins (Existing UX Polish)

### 1.1 Inline Label Toolbar

**Problem**: Adding labels requires memorizing syntax `[category]`. This is the #1 adoption barrier.

**Solution**: When user types `[`, show a floating popup:
- Top section: Recently used categories (last 5)
- Bottom section: All registered types, each with its color dot
- Type to filter (reuse existing autocomplete infrastructure)
- Enter to select, auto-inserts `[category|]` with cursor after `|`
- Escape to dismiss, inserts raw `[`

**Implementation**:
- Modify `editor.rs` - detect `[` keypress in the TextEdit, intercept before egui processes it
- Reuse `AutocompleteMode` enum, add `Category` mode handling (already exists!)
- The popup rendering code is already 80% there in lines 254-322 of editor.rs
- Just need to trigger it on `[` instead of only on typing after `[`

**Files**: `editor.rs` (primary)

### 1.2 Enhanced Outline Sidebar

**Problem**: The outline is visually flat. Hard to distinguish heading levels, label types, and current position at a glance.

**Solution**:
- Color-code each label type with its registered color dot
- Indent headings with proper tree hierarchy lines (using thin vertical lines)
- Show "ghost items" from template with dashed borders and lower opacity
- Auto-scroll the outline to keep the active item visible
- Add a small progress indicator showing reading position (% of document)

**Implementation**:
- Modify `sidebar.rs` Outline branch
- Add color dots from `prefs.label_types` matching
- Calculate scroll position from cursor vs total lines

**Files**: `sidebar.rs` (primary)

### 1.3 Contextual Status Bar

**Problem**: Current status bar shows raw stats (lines/chars/words/labels) - useful but not contextual.

**Solution**: Make it aware of current context:
- **Default**: `Ln 42 | 1,234 words | 23 labels`
- **When cursor on a heading**: `Ln 42 | Section: "第三章 风起" | 456 words in section`
- **When cursor on a label**: `Ln 42 | [角色] 张三 | 3 relations | 5 occurrences`
- Click on the status bar to expand a mini writing-health dashboard

**Implementation**:
- Modify `menus.rs` status bar rendering
- Detect cursor position, check if on heading or label
- Show contextual info from parsed data

**Files**: `menus.rs` (primary), `app.rs` (add helper methods)

---

## Phase 2: Semantic Awareness (Core Differentiation)

### 2.1 Semantic Minimap

**Problem**: Writers can't see the "shape" of their scaffolding without scrolling through the entire document.

**Solution**: A thin vertical strip (~40px) on the right side of the editor:
- Colored dots for each semantic label (color = category)
- Gray bars for headings (height proportional to heading level)
- Semi-transparent rectangle showing current viewport
- Click anywhere to jump to that position
- Scales to full document height

**Implementation**:
- Add a right-side panel in `editor.rs` after the TextEdit
- Render using `ui.painter()` with small colored rectangles
- Calculate positions from label byte offsets → line numbers → y positions
- Handle click events to set cursor position

**Files**: `editor.rs` (primary), `parser.rs` (minimap data helper)

### 2.2 Label Relations Panel

**Problem**: Relations between labels (rf, sp, @) are defined but invisible during writing.

**Solution**: When cursor is on a label, show a small panel below the editor:
- List of related labels as clickable pills: `[角色] 李四 → [关系:师徒]`
- Arrow keys (Alt+Up/Down) to cycle through relations
- Visual: small colored pills showing relation type
- Disappears when cursor moves off a label

**Implementation**:
- Parse relation codes from current label
- Render as a horizontal scrollable area below the TextEdit
- Use existing `parse_semantic_labels` and relation data

**Files**: `editor.rs` (add below TextEdit), `app.rs` (state for relation panel)

### 2.3 Writing Health Indicators

**Problem**: Writers have no feedback on whether their scaffolding is effective.

**Solution**: Simple visual indicators:
- **Label density**: Color-coded per section (green = good, yellow = sparse, red = overloaded)
- **Category balance**: Small horizontal bar chart in sidebar header showing distribution
- **Orphan warning**: Yellow indicator if labels have no relations

**Implementation**:
- Compute density: count labels per N lines, compare to thresholds
- Compute balance: count per category, render as proportional bars
- Compute orphans: labels with no relation codes referencing them

**Files**: `sidebar.rs` (category balance chart), `editor.rs` (density in minimap)

---

## Phase 3: Navigation & Discovery

### 3.1 Quick Navigate (Ctrl+G)

**Problem**: In a 50k-word document, finding a specific label or heading requires scrolling.

**Solution**: A fuzzy-search modal (like Ctrl+P command palette):
- Search across headings AND labels
- Type to filter
- Show: type icon + preview text + line number
- Arrow keys to navigate, Enter to jump
- Separate filters: `@` prefix for labels, `#` for headings, plain text for both

**Implementation**:
- New modal in `menus.rs` following the command palette pattern
- Parse all headings + labels, build searchable list
- Fuzzy match using simple character-sequence matching

**Files**: `menus.rs` (new modal), `app.rs` (state), `main.rs` (Ctrl+G shortcut)

### 3.2 Command Palette Enhancement

**Problem**: The command palette has limited commands.

**Solution**: Add semantic-aware commands:
- "Insert label: [type]" for each registered type
- "Go to label type: X"
- "Toggle semantic minimap"
- "Toggle writing health"
- "Export clean text"
- "Export annotated"
- Show keyboard shortcuts inline with each command

**Files**: `menus.rs` (add commands)

### 3.3 Onboarding & Help

**Problem**: New users don't understand semantic labels.

**Solution**:
- First-run: brief overlay showing label syntax with examples
- Sidebar `?` button: shows label syntax reference
- Tooltips on sidebar mode buttons explaining each view
- "Show keyboard shortcuts" command (Ctrl+/ or F1)

**Files**: `menus.rs` (help overlay), `sidebar.rs` (tooltips), `app.rs` (first-run state)

---

## Phase 4: Export & Polish

### 4.1 Export Preview

**Problem**: Users don't trust the clean export. "Will it look right?"

**Solution**: Before export, show a split view:
- Left: original text with labels highlighted
- Right: clean export result
- Toggle between export formats (clean, flat, annotated)
- Confirm button to actually export

**Files**: `menus.rs` (export modal), `export.rs` (existing logic)

### 4.2 Visual Theme Polish

**Problem**: Default themes feel generic for a writing tool.

**Solution**:
- Craft 3 signature themes:
  - "竹简" (Bamboo): Warm paper tones, bamboo green accents (current default, refined)
  - "墨" (Ink): True dark mode with careful contrast, ink-wash inspired
  - "素" (Plain): Minimal white, single accent color, maximum focus
- Larger default line height (1.6x font size)
- Better paragraph spacing
- Refined tab bar (thinner, more elegant)

**Files**: `parser.rs` (theme presets), `main.rs` (default theme), `menus.rs` (tab bar)

---

## Implementation Priority

| Priority | Feature | Impact | Effort | Phase |
|----------|---------|--------|--------|-------|
| P0 | Inline Label Toolbar | VERY HIGH | Medium | 1 |
| P0 | Enhanced Outline | HIGH | Low | 1 |
| P1 | Contextual Status Bar | HIGH | Low | 1 |
| P1 | Semantic Minimap | VERY HIGH | Medium | 2 |
| P1 | Quick Navigate (Ctrl+G) | HIGH | Medium | 3 |
| P2 | Label Relations Panel | MEDIUM | Medium | 2 |
| P2 | Writing Health Indicators | MEDIUM | Medium | 2 |
| P2 | Command Palette Enhancement | MEDIUM | Low | 3 |
| P3 | Onboarding & Help | MEDIUM | Low | 3 |
| P3 | Export Preview | MEDIUM | Medium | 4 |
| P3 | Visual Theme Polish | LOW-MEDIUM | Medium | 4 |

## Recommended Starting Point

**Start with Phase 1.1 (Inline Label Toolbar)** - This is the single highest-ROI feature. It transforms label creation from a syntax memorization task to a visual interaction. The autocomplete infrastructure already exists; we just need to trigger it differently and enhance it with color dots and recent types.
