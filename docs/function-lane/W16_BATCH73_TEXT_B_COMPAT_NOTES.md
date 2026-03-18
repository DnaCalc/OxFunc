# W16 Batch 73 - Text B Compatibility Functions

Status: `in_progress-provisional`
Workset: `W16`

## Scope
1. `FINDB`
2. `LEFTB`
3. `LENB`
4. `MIDB`
5. `REPLACEB`
6. `RIGHTB`
7. `SEARCHB`

## Current Batch Shape
1. For the current Unicode baseline, these are admitted as compatibility delegates to the non-`B` text functions.
2. The slice is explicitly current-baseline only and does not claim older DBCS-byte-count semantics outside that baseline.

## Pinned Lanes
1. `LENB("A😀B") -> 4`
2. `LEFTB("abcdef",3) -> "abc"`
3. `MIDB("abcdef",2,3) -> "bcd"`
4. `SEARCHB("CD","abcdef") -> 3`

## Open Issues
1. Older DBCS-byte semantics remain a compatibility/version sweep item outside the current phase slice.
