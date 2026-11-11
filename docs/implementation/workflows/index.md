# Workflows

How work is done in this project, following SpecItOut.

## Issues

Issues are reached only through the `sio` client (`sio issues list | show | create | comment …`),
with `SPECITOUT_ACTOR` set. The forge is `none`: issues live on the orphan `issues` branch,
written only by the client. Labels: `formats`, `packs`, `flight`, `combat`, `render`, `input`,
`testbed`, `campaign`, `platform`, `docs`.

## Rapid Prototyping

The project follows the **Rapid Prototyping** workflow (a SpecItOut proposal, P65; not yet
supported by the `sio` client). The [definition](../../definition/index.md) and the
[specification](../../specification/index.md) live on this site, at version 1.0.0 from
2026-11-11.

1. **Each user request**, in the same pass and **before any code**:
    1. a feature (`F-<AREA>-NNN`) is created or updated on the definition page for its area;
    2. requirements (`R-<AREA>-NNN`) citing that feature are written or revised on the
       specification page for the area;
    3. issues are queued in `sio`, citing the requirements.
2. **No approval gates.** The definition and the specification are a permanent record: IDs are
   never reused, and each page's revision history and the
   [change history](../../meta/change-history.md) record every change.
3. **The specification moves before the code**, so no spec sync is needed: code is written
   against requirements that already say what it must do. Requirements written from code that
   predates this workflow are marked *Reverse-engineered: yes*.
4. **When the prototype is complete**, the definition and specification seed a formal
   definition and specification for a **production version** built through the SpecItOut fast
   track.

Until `sio` supports a `prototype` kind, the manifest stays `kind: implementation` with `spec`
pointing at this worktree's `docs/specification` ([DEV-042](../deviations/index.md)).

## Specification revision (before 2026-11-11)

Before the definition and specification were bootstrapped there was no specification
(`specVersion: null`) and work was **implementation-first**: each stage was planned on this site
and pages cited plan stages. Requirements for that work were written afterwards from the code
and are marked *Reverse-engineered: yes*.

## Implementation

1. Pick the next item from the current [stage](../plan/index.md); its issue cites the requirements it implements.
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
