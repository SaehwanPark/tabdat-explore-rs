use Nldata.csv
isid source_row
sql """SELECT source_row AS id,1 AS alternative,dbeach AS chosen,pbeach AS p,qbeach AS q FROM active UNION ALL SELECT source_row,2,dpier,ppier,qpier FROM active UNION ALL SELECT source_row,3,dprivate,pprivate,qprivate FROM active UNION ALL SELECT source_row,4,dcharter,pcharter,qcharter FROM active ORDER BY id,alternative""" into choice_rows
isid id alternative
generate pbase = p
generate qbase = q
clogit chosen p q, group(id) alternative(alternative)
predict pr, pr
sql """SELECT * REPLACE(pbase+CASE WHEN alternative=1 THEN 1 ELSE 0 END AS p,qbase AS q) FROM active ORDER BY id,alternative""" into price1
predict p_price1, pr
sql """SELECT * REPLACE(pbase+CASE WHEN alternative=2 THEN 1 ELSE 0 END AS p,qbase AS q) FROM active ORDER BY id,alternative""" into price2
predict p_price2, pr
sql """SELECT * REPLACE(pbase+CASE WHEN alternative=3 THEN 1 ELSE 0 END AS p,qbase AS q) FROM active ORDER BY id,alternative""" into price3
predict p_price3, pr
sql """SELECT * REPLACE(pbase+CASE WHEN alternative=4 THEN 1 ELSE 0 END AS p,qbase AS q) FROM active ORDER BY id,alternative""" into price4
predict p_price4, pr
sql """SELECT * REPLACE(pbase AS p,qbase+CASE WHEN alternative=1 THEN .001 ELSE 0 END AS q) FROM active ORDER BY id,alternative""" into catch1
predict p_catch1, pr
sql """SELECT * REPLACE(pbase AS p,qbase+CASE WHEN alternative=2 THEN .001 ELSE 0 END AS q) FROM active ORDER BY id,alternative""" into catch2
predict p_catch2, pr
sql """SELECT * REPLACE(pbase AS p,qbase+CASE WHEN alternative=3 THEN .001 ELSE 0 END AS q) FROM active ORDER BY id,alternative""" into catch3
predict p_catch3, pr
sql """SELECT * REPLACE(pbase AS p,qbase+CASE WHEN alternative=4 THEN .001 ELSE 0 END AS q) FROM active ORDER BY id,alternative""" into catch4
predict p_catch4, pr
sql """SELECT * REPLACE(pbase AS p,qbase AS q) FROM active ORDER BY id,alternative""" into restored_choices
export fishing_clogit_predictions.csv
sql """SELECT alternative,avg(pr) AS mean_probability,avg(100*(p_price1-pr)) AS price1,avg(100*(p_price2-pr)) AS price2,avg(100*(p_price3-pr)) AS price3,avg(100*(p_price4-pr)) AS price4,avg(1000*(p_catch1-pr)) AS catch1,avg(1000*(p_catch2-pr)) AS catch2,avg(1000*(p_catch3-pr)) AS catch3,avg(1000*(p_catch4-pr)) AS catch4 FROM active GROUP BY alternative ORDER BY alternative""" into finite_contrasts
head 4
export fishing_clogit_contrasts.csv
