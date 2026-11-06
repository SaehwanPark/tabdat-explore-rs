# Optional historical precision investigation, not runtime or the ASCII oracle.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)<2 || length(args)>3 || (length(args)==3 && args[3]!='--precise'))
  stop('usage: dta_reference.R vietnam_ex1.dta output.csv [--precise]')
d <- foreign::read.dta(args[1], convert.factors=FALSE)
if (nrow(d)!=5999 || anyNA(d[,setdiff(names(d),'lhhex12m')]) || sum(is.na(d$lhhex12m))!=993)
  stop('unexpected DTA source')
x <- data.frame(source_row=seq_len(nrow(d)), lnexp12m=d$lhhex12m,
                age=d$age, sex=d$sex, hhsize=d$hhsize, farm=d$farm,
                educ=d$comped98, lnhhexp=d$lhhexp1, commune=d$commune)
options(digits=17)
if (file.exists(args[2])) stop('output already exists')
# Base R's CSV writer otherwise formats numeric fields with only 15 digits.
# Keep the existing default for prior reproducibility; the optional format
# round-trips the original DTA values through a binary64 CSV reader.
if (length(args)==3) x[] <- lapply(x,function(v)ifelse(is.na(v),NA_character_,sprintf('%.17g',v)))
write.csv(x,args[2],row.names=FALSE,na='')
