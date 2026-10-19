# Independent development-only dnbinom/digamma/trigamma scalar references.
# Rscript nb2_scalar_reference.R > nb2_scalar_reference.csv
options(digits=17)
rows<-list()
for (y in c(0,1,77,1000,100000)) for (a in c(-11,log(.05),log(1.18),11)) {
  r<-exp(-a); mu<-y+.5
  sr<-digamma(y+r)-digamma(r)+log(r/(r+mu))+1-(r+y)/(r+mu)
  rows[[length(rows)+1L]]<-data.frame(y=y,mu=mu,log_alpha=a,
    log_probability=dnbinom(y,size=r,mu=mu,log=TRUE),
    score_eta=r*(y-mu)/(r+mu),score_a=-r*sr,
    information_eta=r*mu*(r+y)/(r+mu)^2,
    information_cross=r*mu*(y-mu)/(r+mu)^2,
    information_a=-r*sr-r^2*(trigamma(y+r)-trigamma(r)+1/r-1/(r+mu)+(y-mu)/(r+mu)^2))
}
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE)
