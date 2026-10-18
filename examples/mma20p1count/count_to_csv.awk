# Author-prepared 18-field token stream, not physical lines. Preserve decimals.
# prepared_row is an export position, NOT a person ID or original DTA row ID.
# Publish a temporary CSV only after conversion succeeds.
BEGIN {
  print "prepared_row,MDU,LC,IDP,LPI,FMDE,PHYSLIM,NDISEASE,HLTHG,HLTHF,HLTHP,LINC,LFAM,EDUCDEC,AGE,FEMALE,CHILD,FEMCHILD,BLACK"
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
    if (count == 18) {
      printf "%d", ++records
      for (j = 1; j <= 18; j++) printf ",%s", value[j]
      printf "\n"
      count = 0
    }
  }
}
END {
  if (!failed && (count != 0 || records == 0)) {
    print "incomplete or empty 18-field count record stream" > "/dev/stderr"
    exit 1
  }
}
