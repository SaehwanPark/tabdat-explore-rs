# Optional historical precision investigation; not runtime or the ASCII oracle.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)<2 || length(args)>3 || (length(args)==3 && args[3]!='--precise'))
  stop('usage: dta_reference.R vietnam_ex2.dta output.csv [--precise]')
d <- foreign::read.dta(args[1],convert.factors=FALSE)
stopifnot(nrow(d)==27766, identical(which(is.na(d$PHARVIS)),27734L),
          all(is.na(d[27734,])))
x <- data.frame(source_row=seq_len(nrow(d)),pharvis=d$PHARVIS,lnhhexp=d$lnhhinc,
                age=d$AGE,sex=d$SEX,married=d$MARRIED,educ=d$COMPED98,
                illness=d$ILLDUM,injury=d$INJDUM,illdays=d$ILLDAYS,
                actdays=d$ACTDAYS,insurance=d$HLTHINS,commune=d$commune)
options(digits=17)
if (file.exists(args[2])) stop('output already exists')
# Preserve the historical default; the optional 17-digit form round-trips
# directly read DTA binary64 values through the native CSV loader.
if (length(args)==3) x[] <- lapply(x,function(v)ifelse(is.na(v),NA_character_,sprintf('%.17g',v)))
write.csv(x,args[2],row.names=FALSE,na='')
