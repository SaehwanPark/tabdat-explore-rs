# Cluster-robust NB2 grouped GOF on audited original-person rows.
# Requires mma20p1count_person.csv produced by prepare_person.R.
use mma20p1count_person.csv
isid prepared_row
isid original_row
isid person_id study_year
nbreg MDU LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK, cluster(person_id)
estat countgof, max(10)
