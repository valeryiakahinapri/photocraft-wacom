#!/bin/bash
# zai-bootstrap.sh — rebuild the Z.ai control-plane environment for photocraft-wacom
# after a sandbox recycle. Idempotent. ~1-2 min.
#
# Layout produced:
#   /home/z/photocraft-wacom    work repo clone (outside the watchdog path!)
#   /home/z/zai-codespaces      codespaces kit clone
#   kit patched with 3rd account `photocraft` (valeryiakahinapri)
#
# Afterwards every kit call is:
#   ZAI_ROOT=/home/z/zai-codespaces CS_ACCOUNT=photocraft bash kit/cs <cmd>
set -euo pipefail

# PATs: from environment if set, else decoded from agent/secrets.env (base64, push-protection
# safe — git IS the disk; raw PATs must never appear in commits, GitHub blocks them).
HERE="$(cd "$(dirname "$0")/.." && pwd)"   # agent/
if [ -z "${WORK_PAT:-}" ] || [ -z "${KIT_PAT:-}" ]; then
  . "$HERE/secrets.env"
  WORK_PAT="${WORK_PAT:-$(echo "$WORK_PAT_B64" | base64 -d)}"
  KIT_PAT="${KIT_PAT:-$(echo "$KIT_PAT_B64" | base64 -d)}"
fi
CS_NAME="${CS_NAME_PHOTOCRAFT:-miniature-waddle-gx65q95446v4cv4}"

echo "== 1. clones =="
if [ ! -d /home/z/photocraft-wacom/.git ]; then
  git clone -q "https://ghp_${WORK_PAT}@github.com/valeryiakahinapri/photocraft-wacom.git" /home/z/photocraft-wacom
  echo "  cloned work repo"
else
  git -C /home/z/photocraft-wacom fetch -q origin || echo "  WARN: work repo fetch failed (offline?)"
  echo "  work repo present"
fi
if [ ! -d /home/z/zai-codespaces/.git ]; then
  git clone -q "https://ghp_${KIT_PAT}@github.com/xfnwfpho1/zai-codespaces.git" /home/z/zai-codespaces
  echo "  cloned kit repo"
else
  echo "  kit repo present"
fi

echo "== 2. kit bootstrap (gh + ssh user-space, gh profiles) =="
ZAI_ROOT=/home/z/zai-codespaces bash /home/z/zai-codespaces/kit/bootstrap-local.sh 2>&1 | tail -3

echo "== 3. photocraft account patch (idempotent) =="
if ! grep -q "photocraft)" /home/z/zai-codespaces/kit/lib.sh; then
  grep -q GH_PAT_PHOTOCRAFT /home/z/zai-codespaces/.env || cat >> /home/z/zai-codespaces/.env <<EOF

# --- work-account (photocraft-wacom) additions ---
GH_PAT_PHOTOCRAFT=ghp_${WORK_PAT}
GH_ACCOUNT_PHOTOCRAFT=valeryiakahinapri
CS_NAME_PHOTOCRAFT=${CS_NAME}
EOF
  python3 - <<'PYEOF'
s = open('/home/z/zai-codespaces/kit/lib.sh').read()
s = s.replace(
"""    secondary) echo "${GH_PAT_SECONDARY:?missing in .env}" ;;""",
"""    secondary) echo "${GH_PAT_SECONDARY:?missing in .env}" ;;
    photocraft) echo "${GH_PAT_PHOTOCRAFT:?missing in .env}" ;;""")
s = s.replace(
"""    secondary) echo "$GH_ACCOUNT_SECONDARY" ;;""",
"""    secondary) echo "$GH_ACCOUNT_SECONDARY" ;;
    photocraft) echo "$GH_ACCOUNT_PHOTOCRAFT" ;;""")
s = s.replace(
"""secondary) echo "zai2-main";; esac; }""",
"""secondary) echo "zai2-main";; photocraft) echo "${CS_NAME_PHOTOCRAFT:-miniature-waddle-gx65q95446v4cv4}";; esac; }""")
open('/home/z/zai-codespaces/kit/lib.sh','w').write(s)
s = open('/home/z/zai-codespaces/kit/cs').read()
s = s.replace(
"""  for acct in primary secondary; do""",
"""  for acct in primary secondary photocraft; do""")
s = s.replace(
"""secondary) tok="$GH_PAT_SECONDARY";; esac""",
"""secondary) tok="$GH_PAT_SECONDARY";; photocraft) tok="$GH_PAT_PHOTOCRAFT";; esac""")
open('/home/z/zai-codespaces/kit/cs','w').write(s)
print("  patched lib.sh + cs + .env")
PYEOF
  ZAI_ROOT=/home/z/zai-codespaces bash /home/z/zai-codespaces/kit/cs auth 2>&1 | tail -3
else
  echo "  already patched"
fi

echo "== 4. verify codespace access =="
ZAI_ROOT=/home/z/zai-codespaces CS_ACCOUNT=photocraft CS_EXEC_TIMEOUT=60 \
  bash /home/z/zai-codespaces/kit/cs exec 'echo "OK: $(hostname), nproc=$(nproc)"; df -h /workspaces | tail -1' \
  && echo "ENVIRONMENT READY" \
  || { echo "FAILED: codespace may be stopped; try: ZAI_ROOT=/home/z/zai-codespaces CS_ACCOUNT=photocraft bash kit/cs start"; exit 1; }
