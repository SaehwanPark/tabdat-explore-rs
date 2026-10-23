use DATA66.csv
keep if nlsflt == 1
ivregress 2sls wage76 black south76 smsa76 reg2 reg3 reg4 reg5 reg6 reg7 reg8 reg9 smsa66 momdad14 sinmom14 nodaded nomomed daded momed famed1 famed2 famed3 famed4 famed5 famed6 famed7 famed8, endog(grade76 exp76 expsq76) iv(col4 age76 agesq76)
estat ar, ci coefficient(grade76)
