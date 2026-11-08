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
| VIS-01 | дерево синтаксичного виводу | 1.5 | |
| IDE-01 | підсвітка в VS Code | 1.0 | |
| REL-02 | релізи, CHANGELOG | 0.5 | |

## Запуск

Потрібен Rust stable.

```bash
cargo run -- run examples/power.sipl X=2 Y=10
```

```
[R ↦ 1024, X ↦ 2, Y ↦ 0]
```

Інші команди: `parse`, `check`, `trace`, усі з `--json`. Див. [`docs/cli.md`](docs/cli.md).

## Перевірки

```bash
cargo test              # усі тести
pwsh scripts/ci.ps1     # те саме, що в CI: формат, clippy, тести, покриття ≥ 80%, пакет
```

Кожна перевірка — окремий скрипт у `scripts/` (PowerShell 7.4+).
Еталони тестів оновлюються командою `SIPL_BLESS=1 cargo test --test fixtures`.

## Структура

```
grammar/      граматика
src/          lexer → parser → ast → interpreter → trace, json, main (CLI)
tests/        модульні, еталонні та інтеграційні тести
spec-tests/   тести специфікації, незалежні від реалізації
examples/     приклади програм
docs/         опис
scripts/      перевірки
```
