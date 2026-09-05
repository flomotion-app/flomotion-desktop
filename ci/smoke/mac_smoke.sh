#!/bin/bash
set -u
EXE="$1"
OUT="smoke-out"
HERE="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$OUT"
codesign --force --sign - "$EXE"

log() { echo "== $*" | tee -a "$OUT/log.txt"; }
act() { "$EXE" act "$@" 2>&1 | tee -a "$OUT/log.txt" | head -c 600; echo; }

log "status before"
"$EXE" status | tee -a "$OUT/log.txt"
log "agent (launches the window and waits for the page)"
"$EXE" agent 2>&1 | tee "$OUT/agent.json" | head -c 400; echo
log "create project and workspace"
act create_project '{"name":"Mac smoke"}'
act create_workspace '{"title":"Box"}'
act upsert_component -f "$HERE/box.json"
sleep 8
log "screen capture of the runner"
screencapture -x "$OUT/screen.png" || log "screencapture failed"
log "client screenshot tool"
"$EXE" act take_screenshot '{"shots":[{"position":[120,-120,90],"target":[0,0,10]}]}' --wait 120 > "$OUT/shot.json" 2>&1
cat "$OUT/shot.json" | head -c 600; echo
python3 - "$OUT" <<'EOF'
import json, shutil, sys, os
out = sys.argv[1]
try:
    d = json.load(open(os.path.join(out, "shot.json")))
except Exception as e:
    print("shot.json not parseable:", e); sys.exit(0)
for f in d.get("files") or []:
    shutil.copy(f, out)
    print("copied", f, os.path.getsize(f), "bytes")
EOF
log "status after"
"$EXE" status | tee -a "$OUT/log.txt"
ls -la "$OUT"
