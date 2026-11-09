import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { createRequire } from 'node:module';
import vsctm from 'vscode-textmate';
import oniguruma from 'vscode-oniguruma';

const require = createRequire(import.meta.url);
const root = new URL('..', import.meta.url);
const examples = new URL('../../examples/', import.meta.url);

const wasm = readFileSync(require.resolve('vscode-oniguruma/release/onig.wasm')).buffer;
await oniguruma.loadWASM(wasm);

const registry = new vsctm.Registry({
  onigLib: Promise.resolve({
    createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
    createOnigString: (text) => new oniguruma.OnigString(text),
  }),
  loadGrammar: async () =>
    vsctm.parseRawGrammar(
      readFileSync(new URL('syntaxes/sipl.tmLanguage.json', root), 'utf8'),
      'sipl.tmLanguage.json',
    ),
});
const grammar = await registry.loadGrammar('source.sipl');

function tokenize(text) {
  const result = [];
  let stack = vsctm.INITIAL;
  for (const line of text.split('\n')) {
    const { tokens, ruleStack } = grammar.tokenizeLine(line, stack);
    for (const token of tokens) {
      const value = line.slice(token.startIndex, token.endIndex);
      if (value.trim()) {
        result.push([value, token.scopes.at(-1)]);
      }
    }
    stack = ruleStack;
  }
  return result;
}

test('while header', () => {
  assert.deepEqual(tokenize('while Y > 0 do'), [
    ['while', 'keyword.control.sipl'],
    ['Y', 'variable.other.sipl'],
    ['>', 'keyword.operator.comparison.sipl'],
    ['0', 'constant.numeric.integer.sipl'],
    ['do', 'keyword.control.sipl'],
  ]);
});

test('assignment and arithmetic', () => {
  assert.deepEqual(tokenize('R := R * X - 1;'), [
    ['R', 'variable.other.sipl'],
    [':=', 'keyword.operator.assignment.sipl'],
    ['R', 'variable.other.sipl'],
    ['*', 'keyword.operator.arithmetic.sipl'],
    ['X', 'variable.other.sipl'],
    ['-', 'keyword.operator.arithmetic.sipl'],
    ['1', 'constant.numeric.integer.sipl'],
    [';', 'punctuation.separator.statement.sipl'],
  ]);
});

test('conditions with unicode and ascii operators', () => {
  assert.deepEqual(tokenize('¬(X = Y) ∨ not X > 1 or skip'), [
    ['¬', 'keyword.operator.logical.sipl'],
    ['(', 'punctuation.section.parens.begin.sipl'],
    ['X', 'variable.other.sipl'],
    ['=', 'keyword.operator.comparison.sipl'],
    ['Y', 'variable.other.sipl'],
    [')', 'punctuation.section.parens.end.sipl'],
    ['∨', 'keyword.operator.logical.sipl'],
    ['not', 'keyword.operator.logical.sipl'],
    ['X', 'variable.other.sipl'],
    ['>', 'keyword.operator.comparison.sipl'],
    ['1', 'constant.numeric.integer.sipl'],
    ['or', 'keyword.operator.logical.sipl'],
    ['skip', 'keyword.control.sipl'],
  ]);
});

test('keywords and blocks', () => {
  assert.deepEqual(
    tokenize('begin if X then skip else skip end').map(([, scope]) => scope),
    [
      'keyword.control.block.sipl',
      'keyword.control.sipl',
      'variable.other.sipl',
      'keyword.control.sipl',
      'keyword.control.sipl',
      'keyword.control.sipl',
      'keyword.control.sipl',
      'keyword.control.block.sipl',
    ],
  );
});

test('keywords inside names are variables', () => {
  assert.deepEqual(tokenize('beginx R1 do_it'), [
    ['beginx', 'variable.other.sipl'],
    ['R1', 'variable.other.sipl'],
    ['do_it', 'variable.other.sipl'],
  ]);
});

test('operators copied from the practicum', () => {
  assert.deepEqual(
    tokenize('R ∗ X − 1 × 2 – 3').map(([, scope]) => scope),
    [
      'variable.other.sipl',
      'keyword.operator.arithmetic.sipl',
      'variable.other.sipl',
      'keyword.operator.arithmetic.sipl',
      'constant.numeric.integer.sipl',
      'keyword.operator.arithmetic.sipl',
      'constant.numeric.integer.sipl',
      'keyword.operator.arithmetic.sipl',
      'constant.numeric.integer.sipl',
    ],
  );
});

test('characters outside SIPL are marked invalid', () => {
  assert.deepEqual(tokenize('X < 1 & 2'), [
    ['X', 'variable.other.sipl'],
    ['<', 'invalid.illegal.sipl'],
    ['1', 'constant.numeric.integer.sipl'],
    ['&', 'invalid.illegal.sipl'],
    ['2', 'constant.numeric.integer.sipl'],
  ]);
});

test('every example has no invalid tokens', () => {
  const files = readdirSync(examples).filter((name) => name.endsWith('.sipl'));
  assert.ok(files.length >= 3);
  for (const file of files) {
    const tokens = tokenize(readFileSync(new URL(file, examples), 'utf8'));
    const invalid = tokens.filter(([, scope]) => scope === 'invalid.illegal.sipl');
    assert.deepEqual(invalid, [], file);
  }
});
