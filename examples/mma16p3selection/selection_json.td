# JSON companion to selection.td; export is intentionally left to the text run.
use mma16p3selection.csv
probit DMED LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK
predict twopart_probability, pr
regress LNMED LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK if DMED == 1
predict twopart_lnmed_xb, xb
heckman LNMED LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK, selectdep(DMED) select(LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK)
predict ml_lnmed_xb, xb
predict ml_selection_probability, pr
predict ml_inverse_mills, mills
predict ml_lnmed_ycond, ycond
heckman LNMED LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK, selectdep(DMED) select(LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK) twostep
predict twostep_lnmed_xb, xb
predict twostep_selection_probability, pr
predict twostep_inverse_mills, mills
predict twostep_lnmed_ycond, ycond
