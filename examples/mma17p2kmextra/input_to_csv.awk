# Extract the executable input block, not inconsistent prose failure labels.
BEGIN { print "source_row,duration,failed" }
{ sub(/\r$/, "") }
/^input duration failed[[:space:]]*$/ {
  if (seen++) {
    print "Expected only one survival input block" > "/dev/stderr"
    bad=1
    exit 1
  }
  block=1
  next
}
block && /^end[[:space:]]*$/ { block=0; done=1; next }
block {
  if (NF!=2 || $1 !~ /^[0-9]+$/ || $1<=0 || $2 !~ /^[01]$/) {
    print "Invalid survival input row " NR > "/dev/stderr"
    bad=1
    exit 1
  }
  count++
  total+=$1
  failures+=$2
  printf "%d,%s,%s\n",count,$1,$2
}
END {
  if (!bad && (!done || block || count!=80 || total!=3170 || failures!=40)) {
    print "Expected one complete80-spell input,40 failures and3170 days" > "/dev/stderr"
    exit 1
  }
}
