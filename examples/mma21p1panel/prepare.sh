#!/bin/sh
# Strict conversion with publication only after success; never replace a target.
set -eu
if [ "$#" -ne 2 ]; then
  echo "usage: sh prepare.sh MOM.dat output.csv" >&2
  exit 1
fi
here=$(CDPATH= cd "$(dirname "$0")" && pwd)
case "$1" in /*) input=$1 ;; *) input="$(pwd)/$1" ;; esac
case "$2" in /*) output=$2 ;; *) output="$(pwd)/$2" ;; esac
if [ -e "$output" ] || [ -L "$output" ]; then
  echo "refusing existing output: $output" >&2
  exit 1
fi
tmp=$(mktemp "$output.tmp.XXXXXX")
trap 'rm -f "$tmp"' 0
trap 'exit 1' 1 2 15
awk -f "$here/mom_to_csv.awk" "$input" > "$tmp"
# A same-directory hard link publishes the completed file without overwriting
# an existing file. The exit trap removes only this invocation's temporary link.
ln "$tmp" "$output"
