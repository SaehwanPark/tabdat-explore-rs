# Extract the ten displayed case1 regression summaries; the caller publishes
# only after the whole log passes the rectangularity checks.
BEGIN { print "imputation,parameter,estimate,standard_error"; current=0; failed=0 }
/Imputation Number=/ {
  value=$0
  sub(/^.*Imputation Number=[ ]*/, "", value)
  sub(/[^0-9].*$/, "", value)
  if (value !~ /^[0-9]+$/ || value+0<1 || value+0>10 || seen_block[value+0]++) {
    print "invalid or duplicate imputation block" > "/dev/stderr"; failed=1; exit 1
  }
  current=value+0
  next
}
current && ($1=="Intercept" || $1=="x1missing" || $1=="x2missing") && $2=="1" {
  if ($3 !~ /^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)$/ ||
      $4 !~ /^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)$/ || $4+0<=0 || seen[current,$1]++) {
    print "invalid or duplicate displayed regression coefficient" > "/dev/stderr"; failed=1; exit 1
  }
  parameter=($1=="Intercept" ? "intercept" : $1)
  printf "%d,%s,%s,%s\n", current,parameter,$3,$4
  count++
}
END {
  if (!failed && count!=30) { print "expected exactly ten complete three-parameter regressions" > "/dev/stderr"; exit 1 }
}
