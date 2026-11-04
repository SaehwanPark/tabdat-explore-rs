# One firm consists of 25 free-format tokens, independent of physical lines.
# ARDSSIC has legitimate Stata '.' missing values; it is not a model variable.
BEGIN {
  print "id,year,cusip,ardssic,scisect,logk,sumpat,logr,pat"
  count = 0
  records = 0
  failed = 0
}
{
  sub(/\r$/, "")
  for (i = 1; i <= NF; i++) {
    token = $i
    position = count + 1
    if (!(position == 2 && token == ".") &&
        (token !~ /^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][+-]?[0-9]+)?$/ ||
         tolower(sprintf("%g", token + 0)) ~ /(inf|nan)/)) {
      print "invalid numeric token at input line " NR ", field " position > "/dev/stderr"
      failed = 1
      exit 1
    }
    value[++count] = token
    if (count == 25) {
      if (value[1] !~ /^[0-9]+([.]0+)?$/ || value[1] + 0 > 9999999) {
        print "invalid CUSIP at record " records + 1 > "/dev/stderr"
        failed = 1
        exit 1
      }
      for (j = 16; j <= 25; j++) {
        if (value[j] !~ /^[0-9]+([.]0+)?$/ || value[j] + 0 > 1000000000) {
          print "invalid patent count at record " records + 1 > "/dev/stderr"
          failed = 1
          exit 1
        }
      }
      id = ++records
      sector = value[2] == "." ? "" : value[2]
      for (j = 5; j <= 9; j++) {
        printf "%d,%d,%s,%s,%s,%s,%s,%s,%s\n", id, 1970+j, value[1], sector,
          value[3], value[4], value[5], value[6+j], value[16+j]
      }
      count = 0
    }
  }
}
END {
  if (!failed && (count != 0 || records == 0)) {
    print "incomplete or empty 25-field patent record stream" > "/dev/stderr"
    exit 1
  }
}
