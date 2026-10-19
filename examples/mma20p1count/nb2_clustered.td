# Native original-person NB2 extension, NOT a printed Table20.5 column.
# prepare_person.R audits original DTA identities against every ASCII model row.
# Assumes independent people; not household/site/multiway or FE/RE inference.
use mma20p1count_person.csv
isid prepared_row
isid original_row
isid person_id study_year
summarize MDU
nbreg MDU LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK, cluster(person_id)
estat margins
predict nb2_person_mu, n
predict nb2_person_xb, xb
export nb2_person_predictions.csv
