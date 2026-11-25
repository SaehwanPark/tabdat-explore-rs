# §20.7 NB2 grouped count goodness-of-fit method workflow.
# Independent-row asymptotics only; source rows are repeated person-years.
use mma20p1count.csv
isid prepared_row
nbreg MDU LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK
estat countgof, max(10)
