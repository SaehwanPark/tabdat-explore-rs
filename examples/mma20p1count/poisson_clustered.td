# Native person-cluster extension of Table20.5 Poisson, not a printed table column.
# prepare_person.R audits ORIGINAL DTA identities against every ASCII model row.
# Requires independent person clusters; not household/site/multiway inference.
use mma20p1count_person.csv
isid prepared_row
isid original_row
isid person_id study_year
summarize MDU
poisson MDU LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK, cluster(person_id)
estat margins
predict poisson_person_mu, n
predict poisson_person_xb, xb
export poisson_person_predictions.csv
