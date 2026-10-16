#!/bin/sh
# Development-only conversion/failed-publication regression; no data download.
set -eu
converter="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)/nldata_to_csv.awk"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT HUP INT TERM
# Four physical lines, Windows endings, and no final newline; one complete person.
printf '4 1 .2 0\r\n0 0 1 1\r\n2 3 4 .1\r\n.2 .3 .4 100' > "$work/valid"
awk -f "$converter" "$work/valid" > "$work/actual"
printf '%s\n%s\n' \
  'source_row,mode,price,crate,dbeach,dpier,dprivate,dcharter,pbeach,ppier,pprivate,pcharter,qbeach,qpier,qprivate,qcharter,income' \
  '1,4,1,.2,0,0,0,1,1,2,3,4,.1,.2,.3,.4,100' > "$work/expected"
cmp "$work/expected" "$work/actual"
for input in '' '1 2 3' '1 2 bad'; do
  printf '%s' "$input" > "$work/invalid"
  printf 'unchanged\n' > "$work/published"
  if awk -f "$converter" "$work/invalid" > "$work/published.tmp" 2> "$work/error" \
    && mv "$work/published.tmp" "$work/published"; then
    printf 'invalid converter input unexpectedly accepted\n' >&2
    exit 1
  fi
  printf 'unchanged\n' > "$work/sentinel"
  cmp "$work/sentinel" "$work/published"
done
printf 'converter: CRLF/final-record preservation and three failure-publication cases passed\n'
