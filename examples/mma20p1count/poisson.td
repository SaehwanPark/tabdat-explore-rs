# Table20.5's first two Poisson columns, not the full count illustration.
# Repeated person-years: independent-row robust is NOT panel/cluster inference.
# Use a fresh output directory; all decimal/design precision is the author's ASCII.
use mma20p1count.csv
isid prepared_row
summarize MDU
poisson MDU LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK
estat margins
predict poisson_classical_mu, n
predict poisson_classical_xb, xb
poisson MDU LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK, robust
estat margins
predict poisson_robust_mu, n
predict poisson_robust_xb, xb
export poisson_predictions.csv
