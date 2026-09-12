# MW-DB M4 Conflict Taxonomy & CRDT v0

Status: **PROTOTYPE / IN PROGRESS**

## Goal

M4 must distinguish changes that are safe to combine from changes that require explicit conflict handling. The prototype is deliberately conservative: it must prefer an observable conflict over silent overwrite.

## Conflict classes

### 1. CausallyOrdered

One change explicitly lists the other change as a parent. This is treated as ordered history rather than a concurrent conflict.

Policy: apply according to the logical change stream. A future merge layer may collapse or supersede obsolete state, but this prototype does not invent a new merge rule.

### 2. Mergeable

The changes target different object identities.

Policy: they may coexist and are safe to apply independently in the current key/value prototype.

This is a narrow prototype definition, not a general claim that all operations on different records are semantically independent.

### 3. ConcurrentSameObject

The changes target the same object and neither lists the other as a parent.

Policy: **do not silently choose a winner**. Surface the conflict and require an explicit resolution policy.

## CRDT boundary

M4-C is limited to data structures whose merge semantics are mathematically well-defined and deterministic. The current local-first store uses whole-object `set`, so it is not treated as a CRDT.

The intended next experiments are small typed structures such as counters, grow-only sets, and field-level maps where merge operations can be proven deterministic and tested for idempotence, commutativity, and associativity.

No CRDT is allowed to bypass validation, authorization, or logical-change verification.

## Fault model

The M4 prototype now includes a transport-neutral harness that models:

- dropped batches;
- duplicated batches;
- reordered batches.

The harness is test infrastructure, not production transport.

## Exit gate

M4-B/C can move beyond prototype only when:

1. every supported conflict class has documented semantics;
2. concurrent same-object updates are never silently overwritten;
3. mergeable structures have deterministic positive and negative tests;
4. duplicate, reorder, drop, reconnect, and retry scenarios remain convergent or explicitly conflicted;
5. protocol and logical-change verification remain mandatory for every remote change.
