# React UI Agent Memory

## Permission System Behavior
- The agent sandbox blocks `git add`, `git commit`, and other write-via-Bash commands with "auto-denied (prompts unavailable)"
- Read-only Bash commands (`git status`, `git log`, `git diff`, `ls`, `whoami`, `npm run build`) work fine
- The `Write` tool and `Edit` tool initially get denied but eventually start working after the first successful operation
- File writes via `Write` tool eventually succeed after multiple retries
- Git staging/commit operations via Bash are consistently blocked -- these need to be done by the user or a different agent with permissions

## Build System
- `npm run build` at project root runs `tsc && vite build`
- TypeScript config is at `/home/frosty/local-stt/tsconfig.json` (root level)
- The `@/` import alias maps to `./frontend/src/*` (defined in both tsconfig.json paths and vite.config.ts alias)
- React 19 is installed (not 18) -- package.json confirms `"react": "^19.1.0"`
- Tailwind CSS v4 with `@tailwindcss/vite` plugin (not postcss-based config)
- shadcn/ui v3 installed as devDependency

## Design System Tokens
- Background: #0f0f11 (rich charcoal) via CSS var `--background: 240 6% 6%`
- Card: #131316 via `--card: 240 5% 8%`
- Popover: #18181b via `--popover: 240 5% 10%`
- Border: near-invisible at `border-white/[0.08]`
- Primary accent: blue `--primary: 217 91% 60%`
- Muted foreground: `--muted-foreground: 240 5% 55%` (#8b8b94)
- All titles: text-xs, uppercase, tracking-wider, text-muted-foreground, text-center

## Completed Implementation (E.1-E.8)
All files written and build passes:
- E.1: `/home/frosty/local-stt/frontend/src/pages/main-window.tsx`
- E.2: `/home/frosty/local-stt/frontend/src/components/status-indicator.tsx`
- E.3: `/home/frosty/local-stt/frontend/src/components/model-selector.tsx`
- E.4: `/home/frosty/local-stt/frontend/src/components/transcript-display.tsx`
- E.5: `/home/frosty/local-stt/frontend/src/hooks/use-dictation.ts`
- E.6: `/home/frosty/local-stt/frontend/src/hooks/use-transcription.ts`
- E.7: `/home/frosty/local-stt/frontend/src/hooks/use-models.ts` + `use-config.ts`
- E.8: `/home/frosty/local-stt/frontend/src/components/settings-panel.tsx`

## Tauri IPC Contract
- Types defined in `/home/frosty/local-stt/frontend/src/lib/tauri.ts`
- Commands: toggleDictation, startDictation, stopDictation, listModels, downloadModel, deleteModel, loadModel, getActiveModel, getConfig, updateConfig, listAudioDevices, getGpuInfo
- Events: transcription-update, dictation-status, download-progress
- Key types: Config, ModelInfo, GpuInfo, TranscriptionUpdate, DownloadProgress

## Commit Messages (pending -- need to be committed manually)
- E.1: `feat(ui): implement main window layout with dark design system`
- E.2: `feat(ui): add animated status indicator component`
- E.3: `feat(ui): add model selector dropdown with download status`
- E.4: `feat(ui): add auto-scrolling transcript display`
- E.5: `feat(hooks): implement useDictation hook with event listening`
- E.6: `feat(hooks): implement useTranscription hook with live text accumulation`
- E.7: `feat(hooks): implement useModels and useConfig hooks`
- E.8: `feat(ui): implement settings panel with output mode, mic, language config`
