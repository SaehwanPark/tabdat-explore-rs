# Development-only base R reference: same pinned 48-state 1995 AER sample.
# Rscript examples/cigarettes_iv/reference_robust_endogenous.R source.csv [expected.csv]
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args) %in% 1:2)
d <- read.csv(args[1]); d <- d[d$year==1995,]; d <- d[order(d$state),]
expected_states <- strsplit(paste("AL AR AZ CA CO CT DE FL GA IA ID IL IN KS KY LA MA MD ME MI MN MO MS MT",
  "NC ND NE NH NJ NM NV NY OH OK OR PA RI SC SD TN TX UT VA VT WA WI WV WY")," ")[[1]]
stopifnot(nrow(d)==48L, identical(d$state,expected_states))
y <- log(d$packs); w <- log(d$income/d$population/d$cpi); x <- log(d$price/d$cpi)
Z <- cbind(1,w,(d$taxs-d$tax)/d$cpi,d$tax/d$cpi)
stopifnot(all(is.finite(c(y,w,x,Z))))
v <- lm.fit(Z,x)$residuals
A <- cbind(1,w,x,v); fit <- lm.fit(A,y); N <- nrow(A); K <- ncol(A)
bread <- solve(crossprod(A)); score <- A*drop(fit$residuals)
correction <- N/(N-K)
covariance <- correction*bread %*% crossprod(score) %*% bread
wald <- fit$coefficients[4]^2/covariance[4,4]
print(list(version=R.version.string,N=N,K=K,df=N-K,correction=correction,
  statistic=wald,p_value=pchisq(wald,1,lower.tail=FALSE)),digits=16)
expected <- data.frame(state=d$state,lpacks=y,lrprice=x,lrincome=w,
  tdiff=Z[,3],rtax=Z[,4])
if(length(args)==2) write.csv(expected,args[2],row.names=FALSE)
