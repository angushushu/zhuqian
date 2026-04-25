# ZhuQian File Standard and Feature Design v2.0

[English] | [简体中文](../zh/FILE_STANDARD.md)

## 1. Design Philosophy

ZhuQian = Markdown + User-defined Semantic Tags + Optional Template Guidance

Core Principles:

2.  **Strict Syntax** — Strictly follows the `semout` standard (uses `[...]`).
3.  **Hidable Tags** — One-click toggle; hiding tags reveals clean content.
4.  **Progressive Enhancement** — Plain Text → Markdown → Semantic Tags → Template Guidance.
5.  **Tool Agnostic** — Readable in any text editor.

---

## 2. File Format

### 2.1 File Structure

```text
---zq-meta---
{
  "template": "freeform",
  "font_size": 16.0,
  "font_name": "times",
  "markdown_render": true,
  "hide_labels": false,
  "language": "En",
  "theme": { "name": "Default", "rules": [...] }
}
---end-meta---

(Body: Markdown + Semantic Tags)
```

*   **Metadata Zone**: From `---zq-meta---` to `---end-meta---` (Legacy support).
*   **Body Zone**: Everything else is the document content.
*   **Extension**: `.md` or `.zq.md` (Recommended).

### 2.2 Body Syntax

Body = Standard Markdown + Semantic Tag Extensions.

#### A) Annotation Tags — Adding semantic types to content

```text
John Doe[Character] pushed open the door.
Beijing[Location] was experiencing heavy rain.
```

| Mode | Display |
| :--- | :--- |
| **Show Labels** | John Doe**[Character]** pushed open the door. (Tag highlighted) |
| **Hide Labels** | John Doe pushed open the door. |

The text before `[]` is the annotated content, and the text inside `[]` is the user-defined type name.

#### B) Independent Tags — Semantic markers not attached to text

```text
[Foreshadowing: Red Umbrella]
```

| Mode | Display |
| :--- | :--- |
| **Show Labels** | **[Foreshadowing: Red Umbrella]** (With color marker) |
| **Hide Labels** | Completely invisible |

Before the colon is the type; after the colon is the value. If there is no colon, the entire string is both the type and the value.

#### C) Note Tags — Writing memos

The system includes one built-in special type: `Note` (`注` in Chinese), used for personal memos during the writing process.

```text
He suddenly stopped in his tracks.[Note: foreshadow a transition here]
```

| Mode | Display |
| :--- | :--- |
| **Show Labels** | He suddenly stopped in his tracks. *[Note: foreshadow a transition here]* |
| **Hide Labels** | He suddenly stopped in his tracks. |

`Note` is the only type recognized by the system to distinguish between "Semantic Annotation" and "Writing Memos." All other tag types are user-defined.

---

## 3. Tag Management System

### 3.1 User-Defined Tag Types

Users manage tag types in settings, where each type includes:

```json
{
  "name": "Character",
  "color": [255, 100, 100],
  "description": "People in the story"
}
```

*   **name** — The type name written inside the delimiters.
*   **color** — The display color for this tag type.
*   **description** — Optional description.

### 3.2 Automatic Tag Registration

When a user uses a previously undefined tag type in the body:

1.  It is automatically registered as a new type.
2.  A default color is selected from a preset palette.
3.  The new category immediately appears in the sidebar.
4.  Users can modify the color and description in settings at any time.

---

## 4. Template System

### 4.1 What is a Template?

Template = **Structural Suggestions**, not mandatory constraints. A template defines:

1.  Suggested heading structure (which sections).
2.  Suggested tag types (users can add/remove/edit).
3.  Validation rules (check for missing sections or tags).

### 4.2 Built-in Templates

#### Freeform Writing
```json
{
  "name": "Freeform",
  "description": "No structural constraints, complete freedom",
  "headings": { "required": [], "optional": [] },
  "label_types": { "recommended": [] }
}
```

#### APA Academic Paper
```json
{
  "name": "APA",
  "description": "APA format academic paper",
  "headings": {
    "required": ["Abstract", "Introduction", "Method", "Results", "Discussion", "References"],
    "optional": ["Appendix", "Acknowledgments"]
  },
  "label_types": {
    "recommended": ["cite", "definition", "claim", "evidence"]
  }
}
```

#### Screenplay
```json
{
  "name": "Screenplay",
  "description": "Film and TV screenplay format",
  "headings": { "required": [], "optional": [] },
  "label_types": {
    "recommended": ["Character", "Scene", "Action", "Dialogue"]
  }
}
```

---

## 5. Functional Modules

### 5.1 Tag Engine (zq-core)
*   Parse tag syntax → TypedLabel (supports custom delimiters).
*   Three tag types: Annotation, Metadata, Note.
*   Tag stripping (hide mode).
*   Tag extraction (sidebar / statistics).

### 5.2 Template Engine (zq-core)
*   Load template definitions.
*   Validate document structure.
*   Calculate completion progress.
*   Manage built-in + custom templates.

### 5.3 Parser & Renderer (zq-core)
*   Markdown parsing.
*   Semantic tag parsing overlay.
*   Theme style mapping.
*   Output StyledSpan.

---

## 6. Progressive Enhancement Levels

```text
Level 0 — Plain Text: Direct writing, no markers.
Level 1 — Markdown: Add headings, bold, lists.
Level 2 — Semantic Tags: Add [User-defined Type] annotations.
Level 3 — Template Guidance: Choose a template, get structural suggestions.
```

Each level is upward compatible. Removing tags from Level 3 = Level 1; removing formatting = Level 0.

---

## 7. Examples

### Novel Writing (Screenplay template, default delimiters)

```text
---zq-meta---
{
  "template": "screenplay",
  "hide_labels": false,
  "language": "En"
}
---end-meta---

# Red Umbrella

## Act I

Li Ming[Character] stands on the rainy street.
He holds a red umbrella[Foreshadowing].

[Note: Create a sense of loneliness here, call back to it later]

A strange woman[Character] walks by, dropping a note[Prop].
```

---

## 8. Implementation Priority

### Phase 1 — Tag System Enhancement
*   Implement three tag types (Annotation / Metadata / Note).
*   Toggle between hiding/showing tags.
*   Automatic tag registration.
*   Custom delimiter support.

### Phase 2 — Tag Management
*   Tag type management panel (add/remove/edit colors).
*   Semantic view in sidebar.
*   Tag statistics.

### Phase 3 — Template System
*   Built-in template definitions.
*   Template validation logic.
*   Template view in sidebar.
*   Custom template support.
