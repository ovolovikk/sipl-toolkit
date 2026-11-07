# Trace виконання SIPL

Код: `src/trace.rs`, хуки в `src/interpreter.rs`.

## Події

| Подія | Коли | Дані |
|---|---|---|
| `before_statement` | перед виконанням `:=`, `if`, `while`, `skip` | вид оператора, позиція |
| `after_statement` | після успішного виконання того самого оператора | вид оператора, позиція |
| `variable_changed` | після присвоювання | ім'я, старе значення (`null`, якщо його не було), нове значення, позиція оператора |
| `condition_checked` | після обчислення умови `if` або кожної перевірки умови `while` | значення, позиція умови |
| `finished` | програма завершилась | кінцевий стан |
| `failed` | помилка виконання | код помилки, повідомлення, позиція |

Для `;` і `begin … end` подій немає: це структура програми, а не дії.

Гарантії, на які можуть спиратися інструменти:

- остання подія завжди `finished` або `failed`, і лише одна з них;
- кожному `after_statement` передує `before_statement` того самого оператора, вкладення
  правильне, як дужки;
- якщо оператор упав, його `after_statement` немає, одразу йде `failed`;
- `condition_checked` для `while` з'являється перед кожною ітерацією і ще раз,
  коли умова стала хибною.

## Як отримати

З коду:

```rust
let events = sipl::trace::trace(&program, state, DEFAULT_ITERATION_LIMIT);
```

Інтерпретатор приймає будь-який `Observer` з одним методом `on_event(Event)`.
`run` використовує порожній `Silent`, `trace` збирає події у `Vec<Event>`.
Відлагоджувач чи візуалізатор може передати власного спостерігача.

З командного рядка:

```bash
sipl trace examples/power.sipl X=2 Y=1
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

Код завершення: `0`, якщо остання подія `finished`, `1`, якщо `failed`.

## JSON

`sipl trace ... --json` виводить один об'єкт в один рядок:

```json
{"events":[
  {"event":"before_statement","statement":"assign","span":{"start":{"line":2,"column":3},"end":{"line":2,"column":9}}},
  {"event":"variable_changed","name":"R","old":null,"new":1,"span":{...}},
  {"event":"after_statement","statement":"assign","span":{...}},
  {"event":"condition_checked","value":false,"span":{...}},
  {"event":"finished","state":{"R":1,"X":2,"Y":0}}
]}
```

(тут розбито на рядки для читабельності, реальний вивід — один рядок).

- `statement`: `assign`, `if`, `while`, `skip`;
- `span`: початок і кінець, `line` і `column` рахуються з 1, `column` — у символах;
  кінець не включається;
- `state`: об'єкт «змінна → число», ключі за абеткою;
- `failed`: `{"event":"failed","error":"undefined_variable","message":"...","span":{...}}`,
  де `error` — один із `undefined_variable`, `overflow`, `iteration_limit_exceeded`.

## Тести

- `src/trace.rs` — точні послідовності подій для `:=`, `if`, `while`, помилки, ліміту
  та точний JSON;
- `tests/fixtures/trace/*.sipl` + `*.in` — програми, `*.trace` — очікуваний текстовий trace;
  для кожної ще перевіряється, що JSON розбирається і закінчується `finished` або `failed`.
