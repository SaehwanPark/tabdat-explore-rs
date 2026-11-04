# Independent synthetic-panel control for the actual script fixture.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=1) stop('usage: Rscript shea_synthetic_reference.R MOMprecise.csv')
d <- read.csv(args[1]); d <- d[order(d$id,d$year),]
stopifnot(nrow(d)==400,length(unique(d$id))==40,all(table(d$id)==10))
current <- d[d$year>=1981,]; lag1 <- d[match(paste(current$id,current$year-1),paste(d$id,d$year)),]
lag2 <- d[match(paste(current$id,current$year-2),paste(d$id,d$year)),]
x <- cbind(dlnwg=current$lnwg-lag1$lnwg,dkids=current$kids-lag1$kids,
 dageh=current$ageh-lag1$ageh,dagesq=current$agesq-lag1$agesq,ddisab=current$disab-lag1$disab)
z <- cbind(lag1$kids,lag1$ageh,lag1$agesq,lag1$disab,
 lag2$ageh,lag2$kids,lag2$agesq,lag2$disab,lag2$lnwg)
residual <- qr.resid(qr(cbind(1,x[,-1])),x[,1])
hat <- qr.fitted(qr(cbind(1,z)),x)
projected <- qr.resid(qr(cbind(1,hat[,-1])),hat[,1])
cat(sprintf('rho=%.17g shea=%.17g\n',cor(residual,projected),cor(residual,projected)^2))
