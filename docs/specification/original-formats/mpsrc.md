---
spec_version: 1.16.0
status: current
updated: 2026-11-11
---

# MPSRC packs

`Animations/mfdanim.mps` is an **MPSRC pack**: a plain bundle of named files. In the original it
holds the cockpit display's animations — 113 Smacker clips (talking heads and small
transmissions) — read in-crate by `formats::mpsrc` since main `319a31b`. The layout below is
*verified* on the reference CD's `mfdanim.mps` (113 entries, all inside the file); the members
are Smacker clips.

## Layout

All numbers little-endian.

| Offset | Size | Field | State |
|---|---|---|---|
| `0x00` | 8 | Signature `MPSRC` followed by three zero bytes | verified |
| `0x08` | 4 | `count`: number of entries | verified |
| `0x0C` | `count` × 136 | Entry table (below) | verified |
| … | … | The members' bytes, at the offsets the entries give | verified |

Each 136-byte entry:

| Offset in entry | Size | Field | State |
|---|---|---|---|
| `0x00` | 128 | Member name, zero-padded (for example a clip name ending `.smk`) | verified |
| `0x80` | 4 | `size` in bytes | verified |
| `0x84` | 4 | `offset` from the start of the file | verified |

The reader rejects a missing signature, a truncated table, an implausible count (over 100 000)
and an entry that runs past the end of the file. Member names are matched without regard to case.

## References

An engine asset reference names a member after `#`: `original:mfdanim.mps#<member>`
(`Content::read_asset`). Such a reference may be used wherever a video is accepted, and as a
voice set's `face` on the comms panel ([Radio voices](../../modding/voices.md#faces-on-the-comms-panel)).

The member names fall into groups by prefix (`at_`, `as_`, `e_`, `w_` and others); **which
speaker each group is, is unknown**, so the `interceptor` campaign does not map faces yet.

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.16.0 | 2026-11-11 | New page: the MPSRC container of `mfdanim.mps`, as read by `formats::mpsrc` (main `319a31b`) |
