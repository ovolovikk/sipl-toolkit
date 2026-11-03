# SIPL Toolkit (Rust)

Курсовий проєкт з дисципліни «Теорія програмування»: інструменти для мови SIPL.

## Реалізовано

| ID | Завдання | Бали |
|---|---|---|
| CORE-01 | Формальна граматика: [`grammar/sipl.ebnf`](grammar/sipl.ebnf), опис у [`docs/grammar.md`](docs/grammar.md) | 0.5 |
| CORE-02 | Лексер, парсер рекурсивним спуском, типізоване AST із `Span` | 1.5 |

## Запуск

Потрібен Rust (stable, edition 2024).

```bash
cargo run -- examples/power.sipl
```

Виводить AST у вигляді S-виразу або помилку з рядком і колонкою:

```
помилка: зайва `;` перед `end`: у SIPL `;` розділяє оператори, а не завершує їх
 --> tests/fixtures/invalid/04_trailing_semicolon.sipl:2:9
  |
2 |   X := 1;
  |         ^
```

## Тести

```bash
cargo test
```

- `tests/fixtures/valid/*.sipl` — коректні програми, очікуване AST у `*.ast`;
- `tests/fixtures/invalid/*.sipl` — некоректні програми, очікувана помилка у `*.err`.

Після свідомої зміни формату виводу еталони перегенеровуються командою
`SIPL_BLESS=1 cargo test --test fixtures`.

## Структура

```
grammar/sipl.ebnf     граматика
src/lexer.rs          текст -> токени
src/parser.rs         токени -> AST
src/ast.rs            вузли AST
src/span.rs           позиції у вихідному тексті
src/diagnostic.rs     повідомлення про помилки
examples/             приклади програм
tests/                тести
```
