## Apex

Apex is a desktop learning platform inspired by Karpathy's knowledge base concept. It lets you organize courses into structured sections, read markdown lessons, and take interactive quizzes — all while tracking your progress.

### Features

- **Course browser** — hierarchical navigation across courses, sections, and content files
- **Markdown viewer** — renders lesson content with CommonMark support
- **Quiz system** — question/answer flashcard-style quizzes with score tracking and review mode
- **Progress tracking** — completed lessons and quizzes are persisted per course
- **Claude integration** — copy command button to open Claude CLI in the course directory

### Tech Stack

- **Rust** (Edition 2024)
- **egui / eframe** — immediate mode GUI framework
- **egui_commonmark** — markdown rendering
- **serde_json** — JSON parsing for quizzes and metadata

### Getting Started

**Option 1: Download**

Download `apex.zip`, unzip it, and run the `apex` executable inside.

On MacOS, I tried moving this to the Applications folder and opening it, which failed. To resolve, run this command:

```bash
xattr -cr /Applications/apex.app
```

**Option 2: Build from source**

```bash
cargo run --release
```

On first launch, click **Initialize Apex** to create the data directory.

### Data Directory

Courses are stored in the platform data directory:

| Platform | Path |
|----------|------|
| macOS    | `~/Library/Application Support/Apex/Apex/` |
| Linux    | `~/.local/share/Apex/` |
| Windows  | `%APPDATA%\Apex\Apex\` |

### Course Structure

```
Apex/
└── CourseName/
    ├── index.json        # section ordering
    ├── progress.json     # completed items
    └── SectionName/
        ├── index.json    # content file ordering
        ├── lesson.md     # markdown lesson
        └── quiz.json     # {"question": "answer", ...}
```
