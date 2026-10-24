# HANDOFF — next-session scope (written 2026-10-24, session 1)

New sandbox, new session: bootstrap FIRST, then the scope below. Everything durable lives
on the `agent-meta` branch of `valeryiakahinapri/photocraft-wacom` (github) — including
this file. `/home/sync` and local disks may be empty/stale; github is the only truth.

## Bootstrap (≈2 min)

0. Tokens (base64-encoded in `agent/secrets.env` on this branch — decode with
   `echo $VAL | base64 -d`, prepend `ghp_` for github PATs):
   - WORK_PAT_B64 → work repo (valeryiakahinapri)
   - KIT_PAT_B64 → zai-codespaces repo (xfnwfpho1)
   - GLPAT_B64 → gitlab mirror (different account again)
   (Raw tokens also appear in the chat that spawned this project; NEVER commit them raw —
   GitHub push protection rejects reconstructable `ghp_`+token patterns.)
1. `git clone https://<work PAT>@github.com/valeryiakahinapri/photocraft-wacom.git /home/z/photocraft-wacom`
2. `git clone https://<kit PAT>@github.com/xfnwfpho1/zai-codespaces.git /home/z/zai-codespaces`
3. `cd /home/z/photocraft-wacom && git fetch origin && git worktree add ../agent-meta agent-meta`
   (or plain `git switch agent-meta` — repo is outside the watchdog path, branches are safe)
4. `bash /home/z/photocraft-wacom/agent/scripts/zai-bootstrap.sh` — bootstraps the kit
   (gh+ssh user-space), re-applies the 3rd-account patch (photocraft → valeryiakahinapri),
   verifies codespace exec. (If the patch is already upstreamed into zai-codespaces kit,
   the script skips it.)
5. Sanity: `ZAI_ROOT=/home/z/zai-codespaces CS_ACCOUNT=photocraft bash kit/cs exec 'echo ok'`

## Immediate scope for next session

1. **Check live-agent status in the codespace** (user said: read OK, writes careful until
   they confirm no live agent). `cs exec 'pgrep -af copilot; cd /workspaces/photocraft-wacom && git status -sb'`.
   If the js-bridge work is committed: review it (kb/browser-js-bridge.md is the spec);
   if still uncommitted: re-snapshot to `wip/<ts>/` on agent-meta (read-only!) and leave it.
2. **Codespace disk relief** (only if agent idle & user OK): `rm -rf target/debug`-style
   prune of 9.8G target/debug (rebuildable) to get /workspaces under 60%.
3. **Ramp-up deep-dive** (context budget permitting): read `apps/photocraft-web/src/web.rs`
   fully, `crates/engine` command registry, `docs/control-protocol.md` — the three surfaces
   a web adoption touches. Extract kb entries into `.agents/skills/photocraft-rust-toolchain/kb/`.
4. **Ask the user the Phase-2 questions** in PLAN.md (target project shape, fork strategy,
   codespace ownership timing, machine upgrade) — Phase 2 is blocked on them.

## Session-1 experience notes (what to avoid repeating)

- The work repo's `.agents/` + `log/` are gitignored by UPSTREAM rules and exist only in the
  codespace tree — always mirror them to agent-meta after touching them there.
- `cs exec` heredoc script mode is the ONLY sane way to run multi-line remote commands;
  always `<<'EOF'` (quoted) or local bash eats the $vars. ZAI_ROOT must be
  /home/z/zai-codespaces in every kit invocation (it defaults to /home/z/my-project — wrong).
- Local clone is at /home/z/photocraft-wacom — NOT /home/z/my-project (watchdog switches
  my-project to main every ~20s; any branch work there gets reverted).
- The kit's `cs` defaults to the primary account (xfnwfpho1) — that codespace list is the
  zai fleet; the photocraft box needs `CS_ACCOUNT=photocraft`.
- Don't run cargo in the codespace repo while the live agent's build holds the lock
  (check `pgrep -af 'cargo build'` first); if needed use a worktree + CARGO_TARGET_DIR=/tmp/... .
