#!/bin/bash
set -u
EXE="$1"
MINUTES="${2:-9}"
OUT="smoke-out"
mkdir -p "$OUT"
codesign --force --sign - "$EXE"

log() { echo "== $*" | tee -a "$OUT/log.txt"; }

timed() {
  local name="$1" input="$2" start end
  start=$(date +%s)
  "$EXE" act "$name" "$input" > "$OUT/last.json" 2>&1
  local code=$?
  end=$(date +%s)
  local ok=yes
  if [ $code -ne 0 ] || grep -qi '"error"\|timed out\|did not' "$OUT/last.json"; then ok=no; fi
  echo "$(( end - T0 )),$name,$(( end - start )),$ok" >> "$OUT/timings.csv"
  echo "$name $(( end - start ))s ok=$ok"
  if [ $ok = no ]; then head -c 400 "$OUT/last.json" | tee -a "$OUT/log.txt"; echo; fi
}

log "launch $("$EXE" --version 2>/dev/null)"
"$EXE" agent > "$OUT/agent.json" 2>&1
"$EXE" act create_project '{"name":"Mac hidden"}' > "$OUT/project.json" 2>&1 || true
grep -q '"role": "manager"' "$OUT/project.json" || { sleep 3; "$EXE" act create_project '{"name":"Mac hidden"}' > "$OUT/project.json" 2>&1; }
head -c 300 "$OUT/project.json"; echo

PID=$(pgrep -n -f "$EXE")
log "hiding window of pid $PID"
osascript -l JavaScript -e "ObjC.import('AppKit'); var a = \$.NSRunningApplication.runningApplicationWithProcessIdentifier($PID); a.hide; 'hidden=' + a.isHidden" | tee -a "$OUT/log.txt"
screencapture -x "$OUT/screen-hidden.png" || true

echo "elapsed_s,command,duration_s,ok" > "$OUT/timings.csv"
T0=$(date +%s)
i=0
while [ $(( $(date +%s) - T0 )) -lt $(( MINUTES * 60 )) ]; do
  i=$((i + 1))
  timed create_workspace "{\"title\":\"Hidden $i\"}"
  timed close_item '{}'
  sleep 20
done

log "diagnose"
"$EXE" diagnose > "$OUT/diagnose.txt" 2>&1
python3 - "$OUT/timings.csv" <<'EOF' | tee "$OUT/summary.txt"
import csv, sys
rows = list(csv.DictReader(open(sys.argv[1])))
for label, part in (("first 5 min", [r for r in rows if int(r["elapsed_s"]) < 300]), ("after 5 min", [r for r in rows if int(r["elapsed_s"]) >= 300])):
    if not part: continue
    d = [int(r["duration_s"]) for r in part]
    bad = [r for r in part if r["ok"] == "no"]
    print(f"{label}: {len(part)} commands, max {max(d)}s, mean {sum(d)/len(d):.1f}s, failed {len(bad)}")
EOF
