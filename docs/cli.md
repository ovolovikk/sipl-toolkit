# Командний рядок

```
sipl parse <файл>                       AST
sipl check <файл>                       лише перевірка синтаксису
sipl run   <файл> [X=1 ...]             кінцевий стан
sipl trace <файл> [X=1 ...]             покрокове виконання
sipl tree  <файл> [--dot | --svg]       дерево виводу, див. tree.md
sipl help
```

`run` і `trace` приймають `--max-iterations N` (за замовчуванням 1 000 000).
Усі команди приймають `--json`.

## Коди завершення

`0` — успіх, `1` — синтаксична помилка, `2` — неправильні аргументи або немає файлу,
`3` — помилка виконання.

## JSON

З `--json` вивід завжди один рядок у stdout, навіть для помилок.

```
parse  {"ok":true,"ast":{...}}
check  {"ok":true}
run    {"ok":true,"state":{"R":1024,"X":2,"Y":0}}
trace  {"ok":true,"events":[...]}        див. trace.md
tree   {"ok":true,"tree":{...}}          див. tree.md
error  {"ok":false,"error":{"kind":"syntax","message":"...","span":{...}}}
```

`kind`: `syntax`, `runtime`, `usage`, `file`. У `runtime` є ще `code`:
`undefined_variable`, `overflow`, `iteration_limit_exceeded`.

Позиція: `"span":{"start":{"line":3,"column":9},"end":{"line":3,"column":10}}`,
рахунок з 1, `end` не включається.

Вузли AST мають `type` і `span`:

```
program   body              if       condition, then, else
assign    variable, value   while    condition, body
sequence  first, second     block    body
skip                        number   value
variable  name              binary   op (+ - *), left, right
compare   op (= >), left, right
or        left, right       not      operand
```

Назви полів і кодів не змінюються, тексти `message` можуть змінюватися.
