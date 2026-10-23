use Nldata.csv
isid source_row
generate ydiv1000 = income/1000
mlogit mode ydiv1000, baseoutcome(1)
predict mnl1, pr outcome(1)
predict mnl2, pr outcome(2)
predict mnl3, pr outcome(3)
predict mnl4, pr outcome(4)
sql """SELECT source_row AS id,1 AS alternative,dbeach AS chosen,pbeach AS p,qbeach AS q,ydiv1000,mnl1 AS mnl_probability FROM active UNION ALL SELECT source_row,2,dpier,ppier,qpier,ydiv1000,mnl2 FROM active UNION ALL SELECT source_row,3,dprivate,pprivate,qprivate,ydiv1000,mnl3 FROM active UNION ALL SELECT source_row,4,dcharter,pcharter,qcharter,ydiv1000,mnl4 FROM active ORDER BY id,alternative""" into combined_choices
isid id alternative
sql """SELECT *,CASE WHEN alternative=2 THEN 1 ELSE 0 END AS dummy2,CASE WHEN alternative=3 THEN 1 ELSE 0 END AS dummy3,CASE WHEN alternative=4 THEN 1 ELSE 0 END AS dummy4,CASE WHEN alternative=2 THEN ydiv1000 ELSE 0 END AS d2y,CASE WHEN alternative=3 THEN ydiv1000 ELSE 0 END AS d3y,CASE WHEN alternative=4 THEN ydiv1000 ELSE 0 END AS d4y,ydiv1000 AS ybase,p AS pbase,q AS qbase FROM active ORDER BY id,alternative""" into explicit_design
clogit chosen dummy2 dummy3 dummy4 d2y d3y d4y, group(id) alternative(alternative)
predict encoded_mnl, pr
clogit chosen p q dummy2 dummy3 dummy4 d2y d3y d4y, group(id) alternative(alternative)
predict baseline, pr
sql """SELECT * REPLACE(ybase+1 AS ydiv1000,dummy2*(ybase+1) AS d2y,dummy3*(ybase+1) AS d3y,dummy4*(ybase+1) AS d4y,pbase AS p,qbase AS q) FROM active ORDER BY id,alternative""" into income_shift
predict income, pr
sql """SELECT * REPLACE(ybase AS ydiv1000,dummy2*ybase AS d2y,dummy3*ybase AS d3y,dummy4*ybase AS d4y,pbase+CASE WHEN alternative=4 THEN 1 ELSE 0 END AS p,qbase AS q) FROM active ORDER BY id,alternative""" into charter_price
predict price, pr
sql """SELECT * REPLACE(ybase AS ydiv1000,dummy2*ybase AS d2y,dummy3*ybase AS d3y,dummy4*ybase AS d4y,pbase AS p,qbase+CASE WHEN alternative=4 THEN .001 ELSE 0 END AS q) FROM active ORDER BY id,alternative""" into charter_catch
predict catch, pr
sql """SELECT * REPLACE(ybase AS ydiv1000,dummy2*ybase AS d2y,dummy3*ybase AS d3y,dummy4*ybase AS d4y,pbase AS p,qbase AS q) FROM active ORDER BY id,alternative""" into restored_combined
export fishing_combined_predictions.csv
sql """SELECT alternative,avg(baseline) AS mean_probability,avg(income-baseline) AS income1000,avg(100*(price-baseline)) AS charter_price1,avg(1000*(catch-baseline)) AS charter_catch001 FROM active GROUP BY alternative ORDER BY alternative""" into combined_contrasts
head 4
export fishing_combined_contrasts.csv
