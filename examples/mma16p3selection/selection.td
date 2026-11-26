# MMA §16.6 RAND, year 2. The ASCII export is already prepared and ordered.
use mma16p3selection.csv
isid source_row
summarize MED DMED LNMED

# Two-part benchmark: a probit for positive spending, then selected-case OLS.
probit DMED LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK
predict twopart_probability, pr
regress LNMED LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK if DMED == 1
predict twopart_lnmed_xb, xb

# Bare heckman is the joint IID Gaussian maximum-likelihood estimator.
heckman LNMED LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK, selectdep(DMED) select(LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK)
predict ml_lnmed_xb, xb
predict ml_selection_probability, pr
predict ml_inverse_mills, mills
predict ml_lnmed_ycond, ycond

# Keep the conventional two-step estimator visibly separate from joint ML.
heckman LNMED LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK, selectdep(DMED) select(LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK) twostep
predict twostep_lnmed_xb, xb
predict twostep_selection_probability, pr
predict twostep_inverse_mills, mills
predict twostep_lnmed_ycond, ycond

# Do not use yexpected as a log-spending mean. See README.md.
export mma16p3selection_predictions.csv
