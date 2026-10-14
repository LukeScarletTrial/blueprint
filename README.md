<p align="center">
  <img src="assets/logo.png" alt="Blueprint Logo" width="140" />
</p>

<h1 align="center">Blueprint</h1>

<p align="center">
  <strong>A modern, readable programming language built for clarity.</strong><br/>
  Square-bracket blocks · Strong typing · Compiles to native executables
</p>

<p align="center">
  <a href="https://github.com/LukeScarletTrial/blueprint/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/github/license/LukeScarletTrial/blueprint?style=flat-square" /></a>
  <a href="https://github.com/LukeScarletTrial/blueprint/raw/main/vscode/blueprint-lang-0.1.0.vsix"><img alt="VS Code Extension" src="https://img.shields.io/badge/download-VS%20Code%20Extension-purple?style=flat-square&logo=visualstudiocode" /></a>
  <a href="https://github.com/LukeScarletTrial/printable/releases"><img alt="Download" src="https://img.shields.io/badge/download-printable%20CLI-blue?style=flat-square" /></a>
</p>

---

## What is Blueprint?

**Blueprint** (`.bl`) is a statically-typed programming language with a clean, minimal syntax. It uses square brackets `[ ]` as code block delimiters — making structure visually obvious at a glance.

Blueprint compiles to **native executables** via the `printable` CLI tool. No VM, no interpreter in production — just a fast binary.

---

## Quick Look

```blueprint
@blueprint import Blueprint
Blueprint.Setup("My App", true)

defstruct Person [
    name: str,
    age: int
]

func IsAdult(person: Person): bool [
    if person.age >= 18 [
        return true
    ] else [
        return false
    ]
]

func Main(): [
    def user = Person { name: "Alex", age: 17 }

    if IsAdult(user) [
        output("Welcome, adult user!")
    ] else [
        output("User is a minor.")
    ]
]
```

**Output:**
```
User is a minor.
```

---

## Get Started

### 1. Download the CLI

Grab the pre-compiled [`printable`](https://github.com/LukeScarletTrial/printable) build tool — no Rust required:

| Platform | Download |
|----------|----------|
| Windows (x64) | [`printable.exe`](https://github.com/LukeScarletTrial/printable/raw/main/printable.exe) |

Place `printable.exe` anywhere in your `PATH`.

### 2. Create a project

```bash
printable init
```

This creates an `app.bls` (Blueprint Setup) configuration file.

### 3. Write your code

Create `main.bl`:

```blueprint
@blueprint import Blueprint
Blueprint.Setup("Hello World", true)

func Main(): [
    output("Hello, Blueprint!")
]
```

### 4. Run or Build

```bash
# Run directly
printable run main.bl

# Compile to a native .exe
printable build main.bl
# → bin/main.exe
```

---

## Language Overview

### Variables

```blueprint
def name = "Blueprint"
def version = 1
def active = true
```

### Functions

```blueprint
func Add(a: int, b: int): int [
    return a + b
]
```

### Structs

```blueprint
defstruct Point [
    x: int,
    y: int
]

def p = Point { x: 10, y: 20 }
output(p.x)
```

### Control Flow

```blueprint
if score >= 90 [
    output("A grade")
] else [
    output("Keep going")
]
```

### Project Config (`app.bls`)

Every Blueprint project has an `app.bls` setup file:

```
[setup]
name = "My App"
version = "1.0.0"
author = "You"

[deps]
# future: declare library dependencies here
```

---

## VS Code Extension

Get syntax highlighting, custom file icons (`.bl` and `.bls`), and snippets for Blueprint.

### Download

Download the packaged extension package directly:

| File | Link |
|------|------|
| **VSIX Package** | [**`blueprint-lang-0.1.0.vsix`**](vscode/blueprint-lang-0.1.0.vsix) |

### Installation

```bash
code --install-extension vscode/blueprint-lang-0.1.0.vsix
```

Or in VS Code: press `Ctrl+Shift+P` → type **Extensions: Install from VSIX...** → select `blueprint-lang-0.1.0.vsix`.

**Features:**
- Syntax highlighting for `.bl` and `.bls` files
- File icons (Blueprint `bL` logo for `.bl`, setup icon for `.bls`)
- Snippets: `func`, `defstruct`, `def`, `ifelse`, `setup`

---

## Project Structure

```
bl/
├── compiler/           # blueprint_compiler library crate (Rust)
│   └── src/
│       ├── lexer.rs    # Logos-based tokeniser
│       ├── ast.rs      # Abstract Syntax Tree
│       ├── parser.rs   # Recursive descent + Pratt parser
│       ├── semantics.rs# Type checking & scope analysis
│       ├── interpreter.rs # Tree-walking interpreter
│       ├── codegen.rs  # Native .exe code generator
│       └── errors.rs   # Unified error types
├── examples/           # Example .bl programs
├── vscode/             # VS Code language extension
├── assets/             # Logos and artwork
└── app.bls             # Blueprint project config
```

---

## Building from Source

Prerequisites: [Rust](https://rustup.rs/) 1.80+, GCC (MSYS2 on Windows)

```bash
# Run all tests
cargo test --workspace

# Build the compiler library
cargo build --release
```

> **Note:** On Windows, ensure the `stable-x86_64-pc-windows-gnu` toolchain is active if you have GCC via MSYS2.

---

## Roadmap

- [x] Lexer with full span tracking
- [x] Recursive descent + Pratt expression parser
- [x] Type checker and semantic analyser
- [x] Tree-walking interpreter
- [x] Native executable code generation
- [x] VS Code extension (syntax, icons, snippets)
- [ ] Standard library (`io`, `math`, `string`)
- [ ] ARC-based memory model
- [ ] Module system and imports
- [ ] Package registry

---

## Related

| Project | Description |
|---------|-------------|
| [`LukeScarletTrial/printable`](https://github.com/LukeScarletTrial/printable) | CLI build tool for Blueprint |

---

## License

[MIT](LICENSE) — © LukeScarletTrial
