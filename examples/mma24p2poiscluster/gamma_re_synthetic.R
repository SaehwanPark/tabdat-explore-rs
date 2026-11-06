# Deterministic, unbalanced ten-slope shared-gamma development fixture.
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==1,!file.exists(a[1]))
set.seed(20261107)
size<-c(6L,1L,1L,4L+(4:48)%%5L);commune<-rep(seq_along(size),size);n<-length(commune)
d<-data.frame(source_row=seq_len(n),pharvis=integer(n),lnhhexp=runif(n,1,4),age=runif(n,2,5),sex=sample(1:2,n,TRUE),married=rbinom(n,1,.5),educ=sample(0:5,n,TRUE),illness=sample(0:3,n,TRUE),injury=rbinom(n,1,.2),illdays=runif(n,0,10),actdays=runif(n,0,4),insurance=rbinom(n,1,.5),commune=commune)
u<-rgamma(length(size),shape=1.2,rate=1.2)
d$pharvis<-rpois(n,u[commune]*exp(-.3+.2*d$lnhhexp-.1*d$insurance+.1*d$sex+.03*d$age+.1*d$married+.02*d$illdays+.03*d$actdays+.1*d$injury+.1*d$illness-.02*d$educ))
d$pharvis[commune%in%c(1,3)]<-0L;d$pharvis[commune==2]<-2L
missing<-d[1,,drop=FALSE];missing[]<-NA;missing$source_row<-n+1L;d<-rbind(d,missing);d<-d[order(d$source_row%%7,-d$source_row),]
options(digits=17);write.csv(d,a[1],row.names=FALSE,quote=FALSE,na='')
cat(nrow(d),'source records;',n,'complete cases;',length(size),'groups\n')
