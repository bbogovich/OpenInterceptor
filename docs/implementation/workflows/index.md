# Workflows

How work is done in this project, following SpecItOut.

## Issues

Issues are reached only through the `sio` client (`sio issues list | show | create | comment …`),
with `SPECITOUT_ACTOR` set. The forge is `none`: issues live on the orphan `issues` branch,
written only by the client. Labels: `formats`, `packs`, `flight`, `combat`, `render`, `input`,
`testbed`, `campaign`, `platform`, `docs`.

## Specification revision

There is no specification yet (`specVersion: null`). Work is **implementation-first**: each
stage is planned on this site, and behaviour that a future specification should own is recorded
here so it can be lifted into requirements. When `OpenInterceptor-Specification` exists, the
manifest pins its version and pages start citing requirement IDs.

## Implementation

1. Pick the next item from the current [stage](../plan/index.md).
2. Write the code with its unit tests and module documentation
   ([commenting standard](../../learning/best-practices/comments.md)).
3. Update the matching [architecture](../architecture/index.md) page *while* implementing.
4. Run `scripts/check.sh`; it must pass.
5. Add manual test cases for anything only a playtest can confirm
   ([manual tests](../testing/manual-tests.md)).
6. Update [implementation status](../status.md) and the
   [revision history](../../reference/revision-history.md).

## Bug resolution

A bug is filed as a `bug` issue with steps to reproduce (scenario ID and ship from the testbed
help). Diagnosis is a comment naming the cause; the fix lands with a unit test that fails before
the fix, or a new manual case when only a playtest can show it.

## A stage is complete when

- Every module has unit tests ([testing policy](../../learning/best-practices/testing-policy.md)),
- every Stage manual case has been run and recorded as passed,
- the testbed has the stage's scenarios,
- and these pages describe what was actually built.
