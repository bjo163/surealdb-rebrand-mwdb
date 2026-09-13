import fs from 'node:fs';

const vectorPath = new URL('../docs/canonical-vectors.json', import.meta.url);
const document = JSON.parse(fs.readFileSync(vectorPath, 'utf8'));

function compareCodePoints(a, b) {
  const left = Array.from(a, (char) => char.codePointAt(0));
  const right = Array.from(b, (char) => char.codePointAt(0));
  const length = Math.min(left.length, right.length);
  for (let i = 0; i < length; i += 1) {
    if (left[i] !== right[i]) return left[i] - right[i];
  }
  return left.length - right.length;
}

function canonicalize(value) {
  if (value === null) return 'null';
  if (typeof value === 'boolean') return value ? 'true' : 'false';
  if (typeof value === 'string') return JSON.stringify(value);

  if (typeof value === 'number') {
    if (!Number.isFinite(value)) throw new Error('non-finite number is not supported');
    if (Object.is(value, -0)) return '0';
    if (Number.isInteger(value) && Math.abs(value) <= Number.MAX_SAFE_INTEGER) {
      return String(value);
    }
    return JSON.stringify(value);
  }

  if (Array.isArray(value)) {
    return `[${value.map(canonicalize).join(',')}]`;
  }

  if (typeof value === 'object') {
    const keys = Object.keys(value).sort(compareCodePoints);
    return `{${keys.map((key) => `${JSON.stringify(key)}:${canonicalize(value[key])}`).join(',')}}`;
  }

  throw new Error(`unsupported JSON value: ${typeof value}`);
}

for (const vector of document.vectors) {
  const actual = canonicalize(vector.input);
  if (actual !== vector.canonical) {
    throw new Error(`vector ${vector.id} failed: expected ${vector.canonical}, got ${actual}`);
  }
}

console.log(`canonical vectors: ${document.vectors.length} passed`);
