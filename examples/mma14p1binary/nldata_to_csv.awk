# Convert the authors' whitespace token stream, 16 fields per person, not lines.
# Write to a temporary destination and rename only after successful conversion.
BEGIN {
  print "source_row,mode,price,crate,dbeach,dpier,dprivate,dcharter,pbeach,ppier,pprivate,pcharter,qbeach,qpier,qprivate,qcharter,income"
  count = 0
  records = 0
  failed = 0
}
{
  sub(/\r$/, "")
  for (i = 1; i <= NF; i++) {
    if ($i !~ /^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][+-]?[0-9]+)?$/) {
      print "invalid numeric token at input line " NR > "/dev/stderr"
      failed = 1
      exit 1
    }
    value[++count] = $i
    if (count == 16) {
      printf "%d", ++records
      for (j = 1; j <= 16; j++) printf ",%s", value[j]
      printf "\n"
      count = 0
    }
  }
}
END {
  if (!failed && (count != 0 || records == 0)) {
    print "incomplete or empty 16-field Nldata record stream" > "/dev/stderr"
    exit 1
  }
}
