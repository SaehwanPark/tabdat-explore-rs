# Strictly convert the source's three fields per physical row; '.' is missing.
# The caller publishes output only after the complete conversion succeeds.
BEGIN { print "source_row,y,x1,x2"; records=0; failed=0 }
{
  sub(/\r$/, "")
  if (NF != 3) {
    print "expected y, x1, x2 at input line " NR > "/dev/stderr"
    failed=1; exit 1
  }
  for (i=1; i<=3; i++) {
    if ($i == "." && i>1) { value[i]=""; continue }
    if ($i !~ /^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][+-]?[0-9]+)?$/ ||
        tolower(sprintf("%g", $i+0)) ~ /(inf|nan)/ || ($i+0)>1000000 || ($i+0)<-1000000) {
      print "invalid, missing outcome or out-of-range number at input line " NR > "/dev/stderr"
      failed=1; exit 1
    }
    value[i]=$i
  }
  if (++records > 10000) { print "input exceeds 10000 rows" > "/dev/stderr"; failed=1; exit 1 }
  printf "%d,%s,%s,%s\n",records,value[1],value[2],value[3]
}
END { if (!failed && records==0) { print "empty input" > "/dev/stderr"; exit 1 } }
