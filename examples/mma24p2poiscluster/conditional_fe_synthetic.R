# Small inspectable deterministic development fixture, not publisher data.
a <- commandArgs(trailingOnly=TRUE)
stopifnot(length(a)==1,!file.exists(a[1]))
set.seed(20261106)
size <- c(8L,1L,6L+(3:19)%%5L)
commune <- rep(1:19,size); n <- length(commune)
d <- data.frame(source_row=seq_len(n),pharvis=integer(n),lnhhexp=runif(n,1,4),
  age=runif(n,2,5),sex=sample(1:2,n,replace=TRUE),married=rbinom(n,1,.5),
  educ=sample(0:5,n,replace=TRUE),illness=sample(0:3,n,replace=TRUE),
  injury=rbinom(n,1,.2),illdays=runif(n,0,10),actdays=runif(n,0,4),
  insurance=rbinom(n,1,.5),commune=commune)
d$pharvis <- rpois(n,exp(-.3+.2*d$lnhhexp-.1*d$insurance+.1*d$sex+.03*d$age+
  .1*d$married+.02*d$illdays+.03*d$actdays+.1*d$injury+.1*d$illness-.02*d$educ+.1*sin(commune)))
d$pharvis[commune==1] <- 0L
for(g in 2:19) if(sum(d$pharvis[commune==g])==0) d$pharvis[which(commune==g)[1]] <- 1L
d$pharvis[commune==2] <- 2L
missing <- d[1,,drop=FALSE]; missing[] <- NA; missing$source_row <- n+1L
d <- rbind(d,missing)
d <- d[order(d$source_row%%7,-d$source_row),]
options(digits=17)
write.csv(d,a[1],row.names=FALSE,quote=FALSE,na='')
cat(nrow(d),'source rows;',n,'complete rows;',sum(size[-1]),'positive-total rows;',length(size)-1,'positive groups\n')
