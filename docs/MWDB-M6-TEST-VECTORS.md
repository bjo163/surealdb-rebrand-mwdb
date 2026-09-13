# MW-DB M6 Canonical Test Vectors

Status: PROTOTYPE VECTORS

These vectors pin the current repository-local canonical JSON boundary. They are intended for independent implementations and are not yet a final standards claim.

## Vector 1 — object key order

Input A:

```json
{"b":2,"a":1}
```

Input B:

```json
{"a":1,"b":2}
```

Canonical bytes for both:

```text
{"a":1,"b":2}
```

## Vector 2 — nested ordering

Input:

```json
{"z":{"b":2,"a":1},"a":[{"d":4,"c":3}]}
```

Canonical bytes:

```text
{"a":[{"c":3,"d":4}],"z":{"a":1,"b":2}}
```

## Vector 3 — array order

Inputs:

```json
[1,2]
[2,1]
```

The canonical bytes must remain different because array ordering is semantic.

## Verification gate

An external implementation passes when it emits exactly the same UTF-8 canonical bytes for all vectors and then produces the same BLAKE3 digest.

The repository does not currently publish frozen hexadecimal digests here because the canonical encoding is still a prototype and numeric normalization has not been frozen across languages.
