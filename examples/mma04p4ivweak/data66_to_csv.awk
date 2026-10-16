# Select the variables used by mma04p4ivweak.do from the authors' fixed-width
# DATA66.dat file. Positions and widths follow DATA66.dct; missing `.` fields
# become empty CSV cells. The original data are not redistributed here.
BEGIN {
  OFS = ","
  names = "wage76 grade76 exp76 expsq76 col4 age76 agesq76 black south76 smsa76 reg2 reg3 reg4 reg5 reg6 reg7 reg8 reg9 smsa66 momdad14 sinmom14 nodaded nomomed daded momed famed1 famed2 famed3 famed4 famed5 famed6 famed7 famed8 nlsflt"
  count = split(names, name, " ")
  starts[1] = 209; widths[1] = 10
  starts[2] = 126; widths[2] = 8
  starts[3] = 219; widths[3] = 8
  starts[4] = 230; widths[4] = 10
  starts[5] = 180; widths[5] = 8
  starts[6] = 243; widths[6] = 8
  starts[7] = 252; widths[7] = 8
  starts[8] = 9; widths[8] = 3
  starts[9] = 207; widths[9] = 1
  starts[10] = 171; widths[10] = 8
  for (i = 0; i < 8; i++) {
    starts[11 + i] = 270 + 9 * i; widths[11 + i] = 8
  }
  starts[19] = 153; widths[19] = 8
  starts[20] = 342; widths[20] = 8
  starts[21] = 351; widths[21] = 8
  starts[22] = 360; widths[22] = 1
  starts[23] = 362; widths[23] = 1
  starts[24] = 365; widths[24] = 10
  starts[25] = 377; widths[25] = 10
  for (i = 0; i < 8; i++) {
    starts[26 + i] = 405 + 9 * i; widths[26 + i] = 8
  }
  starts[34] = 828; widths[34] = 8
  for (i = 1; i <= count; i++) {
    printf "%s%s", name[i], (i == count ? "\n" : OFS)
  }
}
/^[[:space:]]*$/ { next }
{
  for (i = 1; i <= count; i++) {
    value = substr($0, starts[i], widths[i])
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
    if (value == ".") value = ""
    printf "%s%s", value, (i == count ? "\n" : OFS)
  }
}
