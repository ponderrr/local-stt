# Setup Wizard Agent Memory

## Completed Phases
- F.1: Wizard shell (index.tsx) + GPU detection (step-gpu.tsx)
- F.2: Model selection (step-models.tsx)
- F.3: Download progress (step-download.tsx)
- F.4: Completion step (step-complete.tsx) + App.tsx router + App.css deletion

## Key Learnings

### Build Strategy
- The wizard index.tsx imports all 4 step modules at the top level. When creating files incrementally, you MUST create stubs for not-yet-implemented steps so `tsc` passes. Then replace stubs in subsequent phases.
- Build command is `npm run build` from repo root (not `cd frontend && npm run build`). The package.json is at the repo root.

### Path Conventions
- `@/` maps to `frontend/src/` (configured in both tsconfig.json and vite.config.ts)
- Tauri IPC types/wrappers at `@/lib/tauri` -- exports `commands`, `events`, and types: `Config`, `ModelInfo`, `GpuInfo`, `DownloadProgress`, `TranscriptionUpdate`

### TypeScript Strictness
- `noUnusedLocals` and `noUnusedParameters` are enabled. Prefix unused params with `_` in stubs.

### Design System
- Background: #0f0f11 (NOT pure black)
- Cards: #131316 with border-white/[0.08]
- No glassmorphism, no glows, no gradients
- Center all content
- Typography: text-xs uppercase tracking-wider for labels

### Permission Issues
- Tool write permissions may be auto-denied when "prompts unavailable". This is a transient state. Retry in a new conversation turn.
