# Convert the original author ASCII only; preserve row order and GDP precision.
BEGIN { print "source_row,dur,gdp" }
{ sub(/\r$/, "") }
NF == 0 { next }
{
  if (NF != 2 || $1 !~ /^[0-9]+$/ || $1 < 1 || $1 > 235 ||
      $2 !~ /^-?([0-9]+(\.[0-9]*)?|\.[0-9]+)$/) {
    print "Invalid strkdur.asc row " NR > "/dev/stderr"
    bad = 1
    exit 1
  }
  count++
  total += $1
  printf "%d,%s,%s\n", count, $1, $2
}
END {
  if (!bad && (count != 566 || total != 24691)) {
    print "Expected original 566 strikes and 24691 days" > "/dev/stderr"
    exit 1
  }
}
