# W16 Batch 42 - Text Search and Replace Family

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH42-TEXT-SEARCH-REPLACE-20260316`

## Scope
1. `PROPER`
2. `SUBSTITUTE`
3. `REPLACE`
4. `FIND`
5. `SEARCH`

## Native Excel Baseline
Probe artifacts:
1. `.tmp/w16-batch42-text-search-replace-probe.csv`
2. `.tmp/w16-batch42-text-search-replace-edge-probe.csv`

Pinned lanes:
1. `PROPER("hello world") -> "Hello World"`
2. `PROPER("o'brien") -> "O'Brien"`
3. `PROPER("abc123def") -> "Abc123Def"`
4. `SUBSTITUTE("abab","a","x") -> "xbxb"`
5. `SUBSTITUTE("abab","a","x",2) -> "abxb"`
6. `SUBSTITUTE("abab","a","x",0) -> #VALUE!`
7. `SUBSTITUTE("abc","","x") -> "abc"`
8. `REPLACE("abcdef",2,3,"ZZ") -> "aZZef"`
9. `REPLACE("abcdef",7,0,"Z") -> "abcdefZ"`
10. `REPLACE("abc",0,1,"x") -> #VALUE!`
11. `REPLACE("abc",1,-1,"x") -> #VALUE!`
12. `REPLACE(UNICHAR(128512)&"a",2,1,"Z")` replaces the low surrogate only and yields the UTF-16-unit-indexed baseline seen in native Excel.
13. `FIND("b","abc") -> 2`
14. `FIND("B","abc") -> #VALUE!`
15. `FIND("", "abc", 4) -> 4`
16. `FIND("a", UNICHAR(128512)&"a") -> 3`
17. `SEARCH("b","ABC") -> 2`
18. `SEARCH("a?c","axc") -> 1`
19. `SEARCH("a*c","abbbbbc") -> 1`
20. `SEARCH("a~*c","a*c") -> 1`
21. `SEARCH("a~?c","a?c") -> 1`
22. `SEARCH("", "abc", 4) -> #VALUE!`
23. `SEARCH("a", UNICHAR(128512)&"a") -> 3`

## Current Implementation Notes
1. The family is implemented in `text_search_replace_family.rs` and is now wired through the shared dispatch/export surfaces plus the root Lean import.
2. `FIND`, `SEARCH`, and `REPLACE` use one-based UTF-16 code-unit indexing to stay aligned with the existing text-slice helpers and the native emoji probe lanes.
3. `FIND` is case-sensitive and does not treat `*`, `?`, or `~` specially.
4. `SEARCH` is case-insensitive for the current ASCII-seeded baseline and implements `*`, `?`, and `~` wildcard behavior at UTF-16-unit granularity.
5. `SUBSTITUTE` leaves the source text unchanged when `old_text` is empty and rejects `instance_num < 1` with `#VALUE!`.
6. `PROPER` follows the current baseline word-boundary model where non-letter separators, including apostrophes and digits, restart capitalization.
7. The current baseline remains ASCII-seeded for `SEARCH` case folding; broader locale/collation characterization is still separate follow-on evidence work.
