# MOM.dat: eight numeric fields per observation, independent of physical lines.
# Preserve numeric spellings; source_row is original record position, NOT person ID.
# Redirect to a temporary file and publish ONLY after successful conversion.
BEGIN {
  print "source_row,lnhr,lnwg,kids,ageh,agesq,disab,id,year"
  count = 0
  records = 0
  failed = 0
}
{
  sub(/\r$/, "")
  for (i = 1; i <= NF; i++) {
    if ($i !~ /^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][+-]?[0-9]+)?$/ ||
        tolower(sprintf("%g", $i + 0)) ~ /(inf|nan)/) {
      print "invalid or nonfinite numeric token at input line " NR > "/dev/stderr"
      failed = 1
      exit 1
    }
    value[++count] = $i
    if (count == 8) {
      # Source IDs/years have integer or .00 spellings, so reject fractional
      # keys BEFORE floating-point rounding. This is the bounded MOM extract.
      if (value[7] !~ /^[0-9]+([.]0+)?$/ || value[7] + 0 < 1 || value[7] + 0 > 532 ||
          value[8] !~ /^[0-9]+([.]0+)?$/ || value[8] + 0 < 1979 || value[8] + 0 > 1988) {
        print "MOM keys require exact id 1..532 and year 1979..1988 at record " records + 1 > "/dev/stderr"
        failed = 1
        exit 1
      }
      printf "%d", ++records
      for (j = 1; j <= 8; j++) printf ",%s", value[j]
      printf "\n"
      count = 0
    }
  }
}
END {
  if (!failed && (count != 0 || records == 0)) {
    print "incomplete or empty eight-field MOM record stream" > "/dev/stderr"
    exit 1
  }
}
