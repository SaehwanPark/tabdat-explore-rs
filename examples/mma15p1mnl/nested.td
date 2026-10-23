use Nldata.csv
isid source_row
sql """SELECT *,CASE WHEN mode IN (1,2,3,4) AND dbeach=CASE WHEN mode=1 THEN 1 ELSE 0 END AND dpier=CASE WHEN mode=2 THEN 1 ELSE 0 END AND dprivate=CASE WHEN mode=3 THEN 1 ELSE 0 END AND dcharter=CASE WHEN mode=4 THEN 1 ELSE 0 END THEN 1 ELSE 0 END AS source_choice_valid FROM active ORDER BY source_row""" into guarded_source
assert source_choice_valid == 1
generate ydiv1000 = income/1000
sql """SELECT source_row AS id,1 AS alternative,dbeach AS chosen,pbeach AS p,qbeach AS q,ydiv1000 FROM active UNION ALL SELECT source_row,2,dpier,ppier,qpier,ydiv1000 FROM active UNION ALL SELECT source_row,3,dprivate,pprivate,qprivate,ydiv1000 FROM active UNION ALL SELECT source_row,4,dcharter,pcharter,qcharter,ydiv1000 FROM active ORDER BY id,alternative""" into long_choices
isid id alternative
sql """SELECT *,CASE WHEN alternative<=2 THEN 1 ELSE 2 END AS nest,CASE WHEN alternative<=2 THEN 1 ELSE 0 END AS dshore,CASE WHEN alternative<=2 THEN ydiv1000 ELSE 0 END AS dshorey,ydiv1000 AS ybase,p AS pbase,q AS qbase FROM active ORDER BY id,alternative""" into nested_design
clogit chosen p q dshore dshorey, group(id) alternative(alternative)
predict restricted, pr
nlogit chosen p q, group(id) alternative(alternative) nest(nest) upper(dshore dshorey) nonnormalized
predict baseline, pr
sql """SELECT * REPLACE(ybase+1 AS ydiv1000,dshore*(ybase+1) AS dshorey,pbase AS p,qbase AS q) FROM active ORDER BY id,alternative""" into income_shift
predict income, pr
sql """SELECT * REPLACE(ybase AS ydiv1000,dshore*ybase AS dshorey,pbase+CASE WHEN alternative=4 THEN 1 ELSE 0 END AS p,qbase AS q) FROM active ORDER BY id,alternative""" into charter_price
predict price, pr
sql """SELECT * REPLACE(ybase AS ydiv1000,dshore*ybase AS dshorey,pbase AS p,qbase+CASE WHEN alternative=4 THEN .001 ELSE 0 END AS q) FROM active ORDER BY id,alternative""" into charter_catch
predict catch, pr
sql """SELECT * REPLACE(ybase AS ydiv1000,dshore*ybase AS dshorey,pbase AS p,qbase AS q) FROM active ORDER BY id,alternative""" into restored_nested
export fishing_nested_predictions.csv
sql """SELECT alternative,avg(baseline) AS mean_probability,avg(income-baseline) AS income1000,avg(100*(price-baseline)) AS charter_price1,avg(1000*(catch-baseline)) AS charter_catch001 FROM active GROUP BY alternative ORDER BY alternative""" into nested_contrasts
head 4
export fishing_nested_contrasts.csv
