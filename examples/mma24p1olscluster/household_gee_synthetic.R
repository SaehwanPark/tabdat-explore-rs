# Reproducible independent oracle inputs; run the separate dense/geeM reference.
a <- commandArgs(trailingOnly=TRUE)
if(length(a)!=2 || !a[2]%in%c('positive','negative') || file.exists(a[1]))
  stop('usage: household_gee_synthetic.R new-input.csv positive|negative')
set.seed(24051)
sizes <- rep(c(1,2,3,4,5,6),8);group <- rep(seq_along(sizes),sizes)
n <- length(group);X <- matrix(rnorm(n*6),n,6)
individual <- rnorm(n);unit <- rnorm(length(sizes))[group]
if(a[2]=='positive') noise <- 1.3*unit+0.7*individual else {
  means <- as.numeric(rowsum(individual,group))/sizes
  noise <- individual-means[group]+0.3*unit
}
y <- 1.1+drop(X%*%c(.4,-.2,.3,.1,-.3,.2))+noise
source <- data.frame(source_row=seq_len(n+2),lnexp12m=c(y,NA,0),
    lnhhexp=c(X[,1],0,NA),age=c(X[,2],0,0),sex=c(X[,3],0,0),
    hhsize=c(X[,4],0,0),farm=c(X[,5],0,0),educ=c(X[,6],0,0),
    commune=c(group,1,2))
write.csv(source,a[1],row.names=FALSE,na='')
