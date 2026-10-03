# 009 - UCI Time Management

## Goal

Make the engine predictable and usable in UCI GUIs and timed games.

## Scope

- [ ] Parse `go depth`, `nodes`, `movetime`, `wtime`, `btime`, `winc`, `binc`, and `infinite`.
- [ ] Allocate a side-aware time budget with a safety margin.
- [ ] Stop only between completed iterations and return the latest complete PV.
- [ ] Support `stop` and `ponderhit` without duplicate `bestmove` output.
- [ ] Add UCI options for hash size and diagnostic output.

## Tests

Test each `go` form, invalid values, side-to-move selection, immediate stop, and race-free `stop` during a search. Assert that every completed command produces at most one legal `bestmove`.

## Completion

Use the engine in a UCI GUI with clock controls and verify time limits, `stop`, and `infinite` manually as well as in automated tests.
