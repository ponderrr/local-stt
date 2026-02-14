# WhisperType -- Execution Readiness Report

Generated: 2026-02-13
Plan File: `/home/frosty/local-stt/PHASES-FIX-FINAL.md`

---

## EXECUTION ORDER

### Ordered Phase List with Dependencies

```
WAVE 1 (Independent -- can run in parallel):
  Phase 1:  Fix audio capture: device-native format + pipeline resampling  [CRITICAL]
  Phase 4:  Track and join transcription thread on toggle                   [HIGH]
  Phase 2:  Surface output_text errors to frontend                         [HIGH]
  Phase 6:  Set Content Security Policy                                    [MEDIUM]
  Phase 5:  Fix model auto-load race condition                             [MEDIUM]
  Phase 7:  Fix index.html title                                           [LOW]
  Phase 8:  Fix StepComplete placeholder text                              [LOW]
  Phase 9:  Add SAFETY comment to unsafe set_var                           [LOW]
  Phase 10: Fix unnecessary type cast in StepGpu                           [LOW]
  Phase 11: Normalize Config import paths                                  [LOW]
  Phase 13: Remove dead backend stubs                                      [LOW]
  Phase 16: Remove unused shadcn UI components                             [LOW]
  Phase 18: Make model registry static                                     [LOW]

WAVE 2 (Depends on Wave 1 completions):
  Phase 3:  Surface transcription errors to frontend      [depends on Phase 2]
  Phase 14: Remove unused backend methods + hound          [depends on Phase 1]
  Phase 15: Remove dead frontend page + unused state       [depends on Phase 5]
  Phase 12: Remove unused _toggle and isListening          [depends on Phase 2 + 3]

WAVE 3 (Depends on Wave 1):
  Phase 17: Eliminate heap allocation in audio callback    [depends on Phase 1]

WAVE 4 (Depends on ALL):
  Phase E2E: End-to-end acceptance test                    [depends on ALL phases]
```

### Parallelization Analysis

**File conflict groups** (phases that touch the same files and CANNOT run in parallel):

| Group | Phases | Shared Files | Required Order |
|-------|--------|-------------|----------------|
| A: Audio pipeline | 1, 14, 17 | `audio/capture.rs`, `audio/mod.rs` | 1 -> 14 -> 17 |
| B: Dictation hook | 2, 3, 12 | `hooks/use-dictation.ts`, `commands/dictation.rs` | 2 -> 3 -> 12 |
| C: Models hook | 5, 15 | `hooks/use-models.ts` | 5 -> 15 |
| D: Cargo.toml | 14, 17 | `backend/Cargo.toml` | 14 -> 17 |
| E: main-window.tsx | 2, 5, 12 | `pages/main-window.tsx` | 2 -> 12 (5 can be parallel) |
| F: lib.rs | 4, 13 | `backend/src/lib.rs` | Either order, but serial |
| G: tauri.ts | 2, 3 | `frontend/src/lib/tauri.ts` | 2 -> 3 |

**Phases with zero file conflicts** (safe to run in ANY order, parallel with anything):
- Phase 6 (`tauri.conf.json` only)
- Phase 7 (`index.html` only)
- Phase 8 (`step-complete.tsx` only)
- Phase 9 (`main.rs` only)
- Phase 10 (`step-gpu.tsx` only)
- Phase 18 (`transcription/models.rs` only)

### Suggested Agent Allocation

**Optimal: 3 parallel terminals**

```
TERMINAL 1: Backend-Critical Path (longest chain)
  Session 1: Phase 1 (audio resampling) -- COMPLEX, ~30-45 min
  Session 2: Phase 14 (remove dead methods) -- ~5 min
  Session 3: Phase 17 (ring buffer) -- MODERATE, ~20-30 min

TERMINAL 2: Frontend + Error Handling Path
  Session 1: Phase 2 (output errors) -- ~15-20 min
  Session 1: Phase 3 (transcription errors) -- ~10 min (immediately after Phase 2)
  Session 1: Phase 5 (model auto-load) -- ~10 min
  Session 2: Phase 12 (remove _toggle/isListening) -- ~5 min (after Phase 3)
  Session 2: Phase 15 (remove dead frontend) -- ~5 min (after Phase 5)
  Session 2: Phase 16 (remove shadcn UI) -- ~15 min

TERMINAL 3: Quick Fixes (all independent, single session)
  Session 1: Phase 4 (thread join) -- ~15 min
  Session 1: Phase 6 (CSP) -- ~5 min
  Session 1: Phase 7 (index.html title) -- ~1 min
  Session 1: Phase 8 (StepComplete) -- ~1 min
  Session 1: Phase 9 (SAFETY comment) -- ~1 min
  Session 1: Phase 10 (StepGpu cast) -- ~1 min
  Session 1: Phase 11 (normalize imports) -- ~2 min
  Session 1: Phase 13 (remove dead stubs) -- ~5 min
  Session 1: Phase 18 (static registry) -- ~10 min

FINAL (all terminals converge):
  E2E acceptance test -- ~15-20 min
```

**Minimum: 1 serial terminal**

Recommended order for single-agent execution:

```
Session 1 (Critical fixes, ~60 min):
  Phase 1  -> Phase 4 -> Phase 2 -> Phase 3

Session 2 (Medium + quick fixes, ~40 min):
  Phase 5 -> Phase 6 -> Phase 7 -> Phase 8 -> Phase 9 -> Phase 10 -> Phase 11

Session 3 (Dead code removal, ~30 min):
  Phase 12 -> Phase 13 -> Phase 14 -> Phase 15 -> Phase 16

Session 4 (Optimization + validation, ~40 min):
  Phase 17 -> Phase 18 -> E2E
```

### Estimated Session Count

| Configuration | Sessions | Estimated Time |
|---------------|----------|----------------|
| 3 parallel terminals | 2-3 sessions per terminal | ~2-3 hours total wall time |
| 1 serial terminal | 4 sessions | ~3-4 hours total wall time |

---

## FINAL READINESS CHECKLIST

- [x] Every phase has specific file paths and function names
  - Phase 1: `audio/capture.rs` lines 5-8, 11-16, 29-70, 47-51; `audio/mod.rs` lines 42-76, line 54
  - Phase 2: `commands/dictation.rs` line 49; `lib/tauri.ts` line 69; `hooks/use-dictation.ts` lines 7-8; `pages/main-window.tsx` line 13, line 80
  - Phase 3: `commands/dictation.rs` lines 61-63; `lib/tauri.ts`; `hooks/use-dictation.ts`
  - Phase 4: `commands/dictation.rs` lines 9-13, 17-20, 44, 87-93; `lib.rs` lines 19-23
  - Phase 5: `hooks/use-models.ts` lines 4-18, 66; `pages/main-window.tsx` lines 15, 18-28
  - Phase 6: `tauri.conf.json` line 27
  - Phase 7: `index.html` line 7
  - Phase 8: `step-complete.tsx` line 8
  - Phase 9: `main.rs` lines 7-13
  - Phase 10: `step-gpu.tsx` line 15
  - Phase 11: `commands/dictation.rs` line 5; `model_manager/download.rs` line 1; `output/mod.rs` line 4
  - Phase 12: `pages/main-window.tsx` line 13; `hooks/use-dictation.ts` lines 7, 13, 24, 31
  - Phase 13: `model_manager/storage.rs`; `model_manager/mod.rs` line 2; `hotkey/*`; `lib.rs` line 4
  - Phase 14: `audio/capture.rs` lines 72-78; `audio/vad.rs` lines 69-73; `audio/buffer.rs` lines 72-77; `Cargo.toml` line 28
  - Phase 15: `pages/setup.tsx`; `hooks/use-models.ts` lines 7, 24-27, 66
  - Phase 16: `components/ui/*.tsx` (10 files); `package.json` lines 15-16, 27-30
  - Phase 17: `audio/capture.rs`; `audio/mod.rs`; `Cargo.toml`
  - Phase 18: `transcription/models.rs` lines 13-61

- [x] Every phase has at least one gotcha alert or escape hatch
  - Phase 1: 5 gotcha alerts, 5 escape hatches
  - Phase 2: 5 gotcha alerts, 3 escape hatches
  - Phase 3: 4 gotcha alerts, 2 escape hatches
  - Phase 4: 4 gotcha alerts, 3 escape hatches
  - Phase 5: 3 gotcha alerts, 2 escape hatches
  - Phase 6: 3 gotcha alerts, 3 escape hatches
  - Phase 7: 2 gotcha alerts, 1 escape hatch
  - Phase 8: 2 gotcha alerts, 1 escape hatch
  - Phase 9: 2 gotcha alerts, 0 escape hatches (comment-only change)
  - Phase 10: 1 gotcha alert, 1 escape hatch
  - Phase 11: 2 gotcha alerts, 1 escape hatch
  - Phase 12: 3 gotcha alerts, 2 escape hatches
  - Phase 13: 3 gotcha alerts, 2 escape hatches
  - Phase 14: 3 gotcha alerts, 2 escape hatches
  - Phase 15: 2 gotcha alerts, 1 escape hatch
  - Phase 16: 5 gotcha alerts, 2 escape hatches
  - Phase 17: 6 gotcha alerts, 4 escape hatches
  - Phase 18: 3 gotcha alerts, 3 escape hatches

- [x] Every verification step is concretely testable
  - All phases have `cargo check` or `npm run build` as minimum verification
  - Phases 1-4 have specific observable behaviors (audio starts, errors surface, rapid toggling works)
  - Phase 6 has DevTools Console check for CSP violations
  - Dead code phases have file-existence checks
  - E2E has 15 concrete acceptance criteria

- [x] Dependency order verified -- no circular or missing dependencies
  - Phase graph is a DAG with no cycles
  - All forward references resolved: Phase 3 depends on Phase 2 (error mechanism), Phase 14 depends on Phase 1 (capture.rs changes), Phase 15 depends on Phase 5 (use-models.ts), Phase 12 depends on Phases 2+3 (use-dictation.ts), Phase 17 depends on Phase 1 (pipeline architecture)
  - No backward dependencies

- [x] All original audit findings accounted for
  - BUG-1 (16kHz hardcode): Phase 1
  - BUG-2 (thread leak): Phase 4 -- now includes stop_dictation() fix
  - BUG-3 (output errors swallowed): Phase 2
  - BUG-4 (transcription errors stderr only): Phase 3
  - BUG-5 (model auto-load race): Phase 5
  - BUG-6 (CSP null): Phase 6
  - SMELL-1 through SMELL-10: Phases 7-12
  - DEAD code: Phases 13-16
  - OPT-1 (heap alloc callback): Phase 17
  - OPT-2 (Mutex held during inference): Documented in Deferred section
  - OPT-3 (registry allocates): Phase 18
  - Additional finding (inconsistent imports in download.rs and output/mod.rs): Phase 11 now covers ALL three files

- [x] End-to-end acceptance test exists as the final phase
  - Phase E2E with 15 concrete acceptance criteria
  - Includes clean start, setup wizard, model loading, dictation, error handling, rapid toggling, settings persistence, CSP check, build verification

- [x] No phase requires the agent to guess or search for information
  - Phase 1: Single resampling approach chosen (pipeline thread), algorithm provided inline
  - Phase 4: stop_dictation() explicitly included, .take() pattern specified
  - Phase 9: Rewritten from "remove unsafe" to "add SAFETY comment" based on confirmed rustc 1.93.0
  - Phase 11: All three files with inconsistent imports listed explicitly
  - Phase 13: Hotkey directory removal stated definitively (not "consider whether")
  - Phase 14: Coordinates with Phase 1 via "read file FRESH" directive
  - Phase 16: Package names corrected from `@radix-ui/*` to `radix-ui` (unified package)

---

## CRITICAL ISSUES RESOLVED

| Issue from Critique | Resolution in PHASES-FIX-FINAL.md |
|---------------------|-----------------------------------|
| Phase 9 factually wrong (set_var IS unsafe on rustc 1.83+) | Rewritten to "add SAFETY comment" instead of "remove unsafe". Verified rustc 1.93.0 on this system. |
| Phase 1 two contradictory resampling options | Committed to ONE approach: resample in pipeline thread in `audio/mod.rs`. Listed both files. Provided algorithm inline. |
| Phase 1 + Phase 17 conflict | Phase 17 now explicitly states it builds on Phase 1's output, replaces only the cpal-to-pipeline channel, keeps resampling in the pipeline thread unchanged. |
| Phase 4 missing stop_dictation() fix | Added explicit instructions to also join thread in `stop_dictation()` at lines 87-93. Created `join_transcription_thread()` helper. |
| Phase 4 join-blocking risk | Specified `.take()` pattern to avoid holding Mutex during join. Added escape hatch for hung thread. |
| Phase 11 misses download.rs and output/mod.rs | Phase 11 now covers all three files with `config::settings::` imports. |
| Phase 14 coordination with Phase 1 | Phase 14 depends on Phase 1, includes "read file FRESH" directive. |
| Phase 16 wrong package name (@radix-ui/* vs radix-ui) | Corrected to reference `radix-ui` (unified package). |
| Phase 13 vague "consider whether" for hotkey module | Stated definitively: delete entire `hotkey/` directory and remove `pub mod hotkey;` from `lib.rs`. |

---

## REPORT TO USER

WHISPERTYPE PLAN COMPLETE.
Total phases: 19 (18 code phases + 1 E2E acceptance test)
Critical: 2 (Phase 1 + E2E) | High: 3 (Phases 2, 3, 4) | Medium: 3 (Phases 5, 6, 17) | Low: 11 (Phases 7-16, 18)
Estimated execution sessions: 4 (serial) or 2-3 per terminal with 3 parallel terminals
Ready for agent execution.
