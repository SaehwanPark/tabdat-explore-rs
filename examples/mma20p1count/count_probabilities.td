# Native observed/fitted count-frequency analysis of the author-program design.
# Printed Table20.6 NB2 row differs from the authors' November2006 program;
# do NOT claim literal printed-row reproduction. Probabilities assume full laws.
use mma20p1count.csv
isid prepared_row
summarize MDU
poisson MDU LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK
estat countprob, max(10)
predict poisson_mu, n
predict poisson_xb, xb
nbreg MDU LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK
estat countprob, max(10)
predict nb2_mu, n
predict nb2_xb, xb
export count_probability_predictions.csv
