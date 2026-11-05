# SIPL Toolkit (Rust)

Курсовий проєкт з дисципліни «Теорія програмування»: інструменти для мови SIPL.

## Реалізовано

| ID | Завдання | Бали |
|---|---|---|
| CORE-01 | Формальна граматика: [`grammar/sipl.ebnf`](grammar/sipl.ebnf), опис у [`docs/grammar.md`](docs/grammar.md) | 0.5 |
| CORE-02 | Лексер, парсер рекурсивним спуском, типізоване AST із `Span` | 1.5 |
| CORE-03 | Інтерпретатор зі станом і лімітом ітерацій, опис у [`docs/interpreter.md`](docs/interpreter.md) | 2.0 |

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
  очікуваний результат у `*.out`.

Після свідомої зміни формату виводу еталони перегенеровуються командою
`SIPL_BLESS=1 cargo test --test fixtures`.

Покриття тестами (потрібен [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)):

```bash
cargo llvm-cov --summary-only
```

Ядро (`src/` без `main.rs`) покрите на 99% рядків.

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
src/main.rs            командний рядок
examples/              приклади програм
tests/                 тести
docs/                  опис граматики та семантики
```
