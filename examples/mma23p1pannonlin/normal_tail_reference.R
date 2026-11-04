# Independent base-R normal tail grid for the SQL inference's declared z domain.
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==1L, !file.exists(args[1]))
z <- seq(-3,3,by=.1)
write.csv(data.frame(z=z,p_value=2*pnorm(-abs(z))),args[1],row.names=FALSE,quote=FALSE)
