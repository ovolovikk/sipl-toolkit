# SIPL Toolkit (Rust)

[![CI](https://github.com/ovolovikk/sipl-toolkit/actions/workflows/ci.yml/badge.svg)](https://github.com/ovolovikk/sipl-toolkit/actions/workflows/ci.yml)

Курсовий проєкт з дисципліни «Теорія програмування»: інструменти для мови SIPL.

## Обрані завдання

Заявлено 10.5 бала (зараховується не більше 10).

| ID | Завдання | Бали | Стан |
|---|---|---|---|
| CORE-01 | Формальна граматика: [`grammar/sipl.ebnf`](grammar/sipl.ebnf), опис у [`docs/grammar.md`](docs/grammar.md) | 0.5 | готово |
| CORE-02 | Лексер, парсер рекурсивним спуском, типізоване AST із `Span` | 1.5 | готово |
| CORE-03 | Інтерпретатор зі станом і лімітом ітерацій, опис у [`docs/interpreter.md`](docs/interpreter.md) | 2.0 | готово |
| REL-01 | CI: формат, clippy, тести, покриття, збірка пакета на кожен push і PR | 0.5 | готово |
| CORE-04 | Події виконання та trace, текстом і в JSON, опис у [`docs/trace.md`](docs/trace.md) | 1.0 | готово |
| CORE-05 | CLI: `parse`, `check`, `run`, `trace`, JSON-режим | 1.0 | |
| CORE-06 | Conformance suite | 1.0 | |
| VIS-01 | Візуалізація дерева синтаксичного виводу | 1.5 | |
| IDE-01 | VS Code extension: підсвітка синтаксису | 1.0 | |
| REL-02 | GitHub Releases, SemVer, CHANGELOG | 0.5 | |

## Запуск

Потрібен Rust (stable, edition 2024).

Розібрати програму і вивести AST у вигляді S-виразу:

```bash
cargo run -- parse examples/power.sipl
```

```
(; (:= R 1) (while (> Y 0) (begin (; (:= R (* R X)) (:= Y (- Y 1))))))
```

Виконати програму на початковому стані:

```bash
cargo run -- run examples/power.sipl X=2 Y=10
```

```
[R ↦ 1024, X ↦ 2, Y ↦ 0]
```

Ліміт ітерацій циклів задається `--max-iterations N` (за замовчуванням 1 000 000).

Покроково показати виконання (додайте `--json`, щоб отримати trace одним JSON-рядком):

```bash
cargo run -- trace examples/power.sipl X=2 Y=1
```

```
2:3    R := 1                R: — → 1
3:3    while Y > 0 do
3:9      Y > 0 = true
5:5      R := R * X            R: 1 → 2
6:5      Y := Y - 1            Y: 1 → 0
3:9      Y > 0 = false
       [R ↦ 2, X ↦ 2, Y ↦ 0]
```

Помилки виводяться з рядком і колонкою:

```
помилка: змінна `Y` не має значення в поточному стані
 --> examples/power.sipl:3:9
  |
3 |   while Y > 0 do
  |         ^
```

Коди завершення: `0` — успіх, `1` — помилка в програмі SIPL, `2` — неправильні аргументи
або файл не читається.

## Тести

```bash
cargo test
```

- `tests/fixtures/valid/*.sipl` — коректні програми, очікуване AST у `*.ast`;
- `tests/fixtures/invalid/*.sipl` — некоректні програми, очікувана помилка у `*.err`;
- `tests/fixtures/run/*.sipl` — програми для виконання: початковий стан у `*.in`,
  очікуваний результат у `*.out`;
- `tests/fixtures/trace/*.sipl` — програми з початковим станом у `*.in`, очікуваний trace у `*.trace`.

Після свідомої зміни формату виводу еталони перегенеровуються командою
`SIPL_BLESS=1 cargo test --test fixtures`.

## Перевірки та CI

Кожна перевірка — окремий скрипт PowerShell 7.4+ у `scripts/`. Їх запускає GitHub Actions
([`.github/workflows/ci.yml`](.github/workflows/ci.yml)) на кожен push у `main` і на кожен PR,
і їх так само можна запустити локально з будь-якої теки:

| Скрипт | Що перевіряє |
|---|---|
| `scripts/format.ps1` | форматування (`cargo fmt --check`) |
| `scripts/lint.ps1` | типи та lint (`cargo clippy`, попередження = помилки) |
| `scripts/test.ps1` | усі тести |
| `scripts/coverage.ps1` | покриття ядра (`src/` без `main.rs`) не менше 80% рядків |
| `scripts/package.ps1` | збірка пакета `.crate` |
| `scripts/ci.ps1` | усе вищенаведене по черзі |

```bash
pwsh scripts/ci.ps1
```

Для покриття потрібен [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov).
Зараз ядро покрите на 99% рядків.

## Структура

```
grammar/sipl.ebnf      граматика
src/lexer.rs           текст -> токени
src/parser.rs          токени -> AST
src/ast.rs             вузли AST
src/span.rs            позиції у вихідному тексті
src/diagnostic.rs      повідомлення про помилки
src/state.rs           стан: змінна -> число
src/interpreter.rs     виконання AST на стані
src/trace.rs           події виконання, trace текстом і в JSON
src/main.rs            командний рядок
examples/              приклади програм
tests/                 тести
docs/                  опис граматики та семантики
scripts/               перевірки (PowerShell 7.4+)
.github/workflows/     CI
```
