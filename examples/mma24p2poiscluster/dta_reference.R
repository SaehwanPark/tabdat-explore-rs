# Optional historical precision investigation; not runtime or the ASCII oracle.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=2) stop('usage: dta_reference.R vietnam_ex2.dta output.csv')
d <- foreign::read.dta(args[1],convert.factors=FALSE)
stopifnot(nrow(d)==27766, identical(which(is.na(d$PHARVIS)),27734L),
          all(is.na(d[27734,])))
x <- data.frame(source_row=seq_len(nrow(d)),pharvis=d$PHARVIS,lnhhexp=d$lnhhinc,
                age=d$AGE,sex=d$SEX,married=d$MARRIED,educ=d$COMPED98,
                illness=d$ILLDUM,injury=d$INJDUM,illdays=d$ILLDAYS,
                actdays=d$ACTDAYS,insurance=d$HLTHINS,commune=d$commune)
options(digits=17)
if (file.exists(args[2])) stop('output already exists')
write.csv(x,args[2],row.names=FALSE,na='')
