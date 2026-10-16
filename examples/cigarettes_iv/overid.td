use CigarettesSW.csv
sql """SELECT state, ln(packs) AS lpacks, ln(price/cpi) AS lrprice, ln(income/population/cpi) AS lrincome, (taxs-tax)/cpi AS tdiff, tax/cpi AS rtax FROM active WHERE year = 1995 ORDER BY state""" into cigarettes_1995
ivregress 2sls lpacks lrincome, endog(lrprice) iv(tdiff rtax)
estat overid
