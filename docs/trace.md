# Trace

Під час виконання інтерпретатор повідомляє про події:

| Подія | Коли |
|---|---|
| `before_statement` / `after_statement` | до й після `:=`, `if`, `while`, `skip` |
| `variable_changed` | після присвоювання: ім'я, старе значення (`null`, якщо не було), нове |
| `condition_checked` | після кожної перевірки умови `if` / `while` |
| `finished` / `failed` | остання подія: кінцевий стан або помилка |

Для `;` і `begin … end` подій немає. Якщо оператор упав, його `after_statement` немає.

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

З `--json` — один рядок:

```
{"ok":true,"events":[{"event":"before_statement","statement":"assign","span":{...}},
 {"event":"variable_changed","name":"R","old":null,"new":1,"span":{...}}, ...,
 {"event":"finished","state":{"R":2,"X":2,"Y":0}}]}
```

У коді: `sipl::trace::trace(&program, state, limit)` повертає `Vec<Event>`.
Інтерпретатор приймає будь-якого `Observer`, тож відлагоджувач може підключити свого.
