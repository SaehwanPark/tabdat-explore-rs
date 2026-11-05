# Optional historical precision investigation, not runtime or the ASCII oracle.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=2) stop('usage: dta_reference.R vietnam_ex1.dta output.csv')
d <- foreign::read.dta(args[1], convert.factors=FALSE)
if (nrow(d)!=5999 || anyNA(d[,setdiff(names(d),'lhhex12m')]) || sum(is.na(d$lhhex12m))!=993)
  stop('unexpected DTA source')
x <- data.frame(source_row=seq_len(nrow(d)), lnexp12m=d$lhhex12m,
                age=d$age, sex=d$sex, hhsize=d$hhsize, farm=d$farm,
                educ=d$comped98, lnhhexp=d$lhhexp1, commune=d$commune)
options(digits=17)
if (file.exists(args[2])) stop('output already exists')
write.csv(x,args[2],row.names=FALSE,na='')
