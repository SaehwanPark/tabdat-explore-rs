#!/bin/sh
# Run after cargo build: argument 1 is readstat-sys OUT_DIR/readstat, already
# checksum-verified and patched by build.rs. Native C only, not Rust instrumentation.
set -eu
source_dir=$1
output_dir=$2
sanitizers=${SANITIZERS:-undefined}
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
mkdir -p "$output_dir"
grep -q 'got_table' "$source_dir/src/stata/readstat_dta_read.c"
iconv=
if [ "$(uname -s)" = Darwin ]; then iconv=-liconv; fi
for unit in readstat_parser readstat_io_unistd; do
  "${CC:-cc}" -std=c99 -Werror -pedantic-errors -Wno-strict-prototypes -O1 -g \
    "-fsanitize=$sanitizers" -fno-omit-frame-pointer -I "$source_dir/src" \
    -Dcalloc=tabdat_fault_calloc -Dfree=tabdat_fault_free \
    -c "$source_dir/src/$unit.c" -o "$output_dir/$unit.o"
done
"${CC:-cc}" -std=c99 -Werror -pedantic-errors -Wno-strict-prototypes -O1 -g \
  "-fsanitize=$sanitizers" -fno-omit-frame-pointer -I "$source_dir/src" \
  "$source_dir/src/CKHashTable.c" "$output_dir/readstat_parser.o" \
  "$source_dir/src/readstat_metadata.c" "$source_dir/src/readstat_variable.c" \
  "$source_dir/src/readstat_value.c" "$output_dir/readstat_io_unistd.o" \
  "$source_dir/src/readstat_convert.c" "$source_dir/src/readstat_bits.c" \
  "$source_dir/src/readstat_error.c" "$source_dir/src/readstat_malloc.c" \
  "$source_dir/src/stata/readstat_dta.c" "$source_dir/src/stata/readstat_dta_read.c" \
  "$source_dir/src/stata/readstat_dta_parse_timestamp.c" \
  "$root/readstat-sys/value_bridge.c" "$root/fixtures/native_boundary_probe.c" \
  -lz $iconv -o "$output_dir/native-probe"
UBSAN_OPTIONS=halt_on_error=1 "$output_dir/native-probe" \
  "$root/fixtures/labeled.dta" "$root/fixtures/patients.dta" \
  "$output_dir/does-not-exist.dta"
