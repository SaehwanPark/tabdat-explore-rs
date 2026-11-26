BEGIN {
  columns = "DMED MED LNMED LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK"
  expected = split(columns, names, " ")
  if (expected != 20) exit 2
  token_count = 0
  record_count = 0
  invalid = 0
}

{
  for (field = 1; field <= NF; field++) {
    token = $field
    sub(/\r$/, "", token)
    if (token != "." && token !~ /^[-+]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][-+]?[0-9]+)?$/) {
      invalid = 1
      continue
    }
    token_count++
    record = int((token_count - 1) / expected) + 1
    position = (token_count - 1) % expected + 1
    values[record, position] = token
    if (position == expected) record_count++
  }
}

END {
  if (invalid || token_count != 111480 || record_count != 5574 || token_count % expected != 0) {
    print "mma16p3selection.asc must contain the pinned 5,574 x 20 numeric token stream" > "/dev/stderr"
    exit 1
  }
  printf "source_row"
  for (field = 1; field <= expected; field++) printf ",%s", names[field]
  printf "\n"
  for (record = 1; record <= record_count; record++) {
    printf "%d", record
    for (field = 1; field <= expected; field++) {
      token = values[record, field]
      if (token == ".") token = ""
      printf ",%s", token
    }
    printf "\n"
  }
}
