# Fixed author-export conversion, not a replacement simulation. Buffer until
# validated so malformed input emits no plausible partial CSV. Preserve tokens.
BEGIN { FS="[ \t]+"; valid=1; count=0 }
{
  sub(/^[ \t]+/, ""); sub(/[ \t\r]+$/, "")
  if (NF != 6) { valid=0; next }
  count++
  line=count
  for (i=1; i<=6; i++) {
    token=$i
    if (token=="." && i==4) token=""
    else if (token !~ /^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][+-]?[0-9]+)?$/) valid=0
    line=line "," token
  }
  output[count]=line
}
END {
  if (!valid || count!=200) {
    print "expected the200-row, six-field MMA16P1TOBIT export" > "/dev/stderr"
    exit 1
  }
  print "source_row,e,lnwage,ystar,ytrunc,ycens,dy"
  for (row=1; row<=count; row++) print output[row]
}
