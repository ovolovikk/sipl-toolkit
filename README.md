# SIPL Toolkit

[![CI](https://github.com/ovolovikk/sipl-toolkit/actions/workflows/ci.yml/badge.svg)](https://github.com/ovolovikk/sipl-toolkit/actions/workflows/ci.yml)

Курсовий проєкт з «Теорії програмування»: парсер, інтерпретатор і trace для мови SIPL, на Rust.

## Завдання

Заявлено 10.5 бала (зараховується не більше 10).

| ID | Що | Бали | |
|---|---|---|---|
| CORE-01 | граматика, [`docs/grammar.md`](docs/grammar.md) | 0.5 | ✓ |
| CORE-02 | парсер і AST | 1.5 | ✓ |
| CORE-03 | інтерпретатор, [`docs/interpreter.md`](docs/interpreter.md) | 2.0 | ✓ |
| CORE-04 | trace, [`docs/trace.md`](docs/trace.md) | 1.0 | ✓ |
| CORE-05 | командний рядок, [`docs/cli.md`](docs/cli.md) | 1.0 | ✓ |
| CORE-06 | тести специфікації, [`spec-tests/`](spec-tests/README.md) | 1.0 | ✓ |
| REL-01 | CI | 0.5 | ✓ |
| VIS-01 | дерево синтаксичного виводу, [`docs/tree.md`](docs/tree.md) | 1.5 | ✓ |
| IDE-01 | підсвітка в VS Code, [`vscode-sipl/`](vscode-sipl/README.md) | 1.0 | ✓ |
| REL-02 | релізи, [`CHANGELOG.md`](CHANGELOG.md) | 0.5 | ✓ |

## Запуск

Потрібен Rust stable.

```bash
cargo run -- run examples/power.sipl X=2 Y=10
```

```
[R ↦ 1024, X ↦ 2, Y ↦ 0]
```

Інші команди: `parse`, `check`, `trace`, `tree` (дерево виводу, також `--dot` і `--svg`), усі з `--json`. Див. [`docs/cli.md`](docs/cli.md).

Розширення для VS Code: `pwsh scripts/extension.ps1` збирає `vscode-sipl/sipl-0.1.0.vsix`,
далі `code --install-extension vscode-sipl/sipl-0.1.0.vsix`. Потрібен Node.js 22+.

## Перевірки

```bash
cargo test              # усі тести
pwsh scripts/ci.ps1     # те саме, що в CI: формат, clippy, тести, покриття ≥ 80%, пакет, розширення
```

Кожна перевірка — окремий скрипт у `scripts/` (PowerShell 7.4+).

Реліз: додати версію в `CHANGELOG.md` і `Cargo.toml`, потім `git tag v0.1.0 && git push origin v0.1.0`.
GitHub сам збере `sipl` для Linux, macOS, Windows і `.vsix` та створить реліз з нотатками з CHANGELOG.
Еталони тестів оновлюються командою `SIPL_BLESS=1 cargo test --test fixtures`.

## Структура

```
grammar/      граматика
src/          lexer → parser → ast → interpreter → trace, parse_tree, json, main (CLI)
tests/        модульні, еталонні та інтеграційні тести
spec-tests/   тести специфікації, незалежні від реалізації
examples/     приклади програм
vscode-sipl/  розширення для VS Code
docs/         опис
scripts/      перевірки
```
