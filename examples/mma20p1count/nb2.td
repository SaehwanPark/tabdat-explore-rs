# Table20.5 NB2 coefficients / model-based and independent-row robust inference.
# Same author ASCII/all-year sample as poisson.td; NOT panel-robust inference.
# Use a fresh session/output directory; no author IDs are present in the ASCII.
use mma20p1count.csv
isid prepared_row
summarize MDU
nbreg MDU LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK
estat margins
predict nb2_classical_mu, n
predict nb2_classical_xb, xb
nbreg MDU LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK, robust
estat margins
predict nb2_robust_mu, n
predict nb2_robust_xb, xb
export nb2_predictions.csv
