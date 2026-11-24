# General normal-reference linear combinations after retained Poisson/NB2 fits.
# The source is repeated person-years; this workflow uses model/row-robust V only.
use mma20p1count.csv
isid prepared_row
poisson MDU LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK
lincom LC
lincom 2 + LC - IDP
lincom LC + IDP - 0.25
poisson MDU LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK, robust
lincom LC
lincom 2 + LC - IDP
lincom LC + IDP - 0.25
nbreg MDU LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK
lincom LC
lincom 2 + LC - IDP
lincom LC + IDP - 0.25
nbreg MDU LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK, robust
lincom LC
lincom 2 + LC - IDP
lincom LC + IDP - 0.25
