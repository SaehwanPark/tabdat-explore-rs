# Development-only base R independent SVD/lm.fit score reference; no packages.
# Rscript examples/cigarettes_iv/reference_robust_overid.R source.csv [expected.csv]
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args) %in% 1:2)
d <- read.csv(args[1]); d <- d[d$year==1995,]; d <- d[order(d$state),]
expected_states <- strsplit(paste("AL AR AZ CA CO CT DE FL GA IA ID IL IN KS KY LA MA MD ME MI MN MO MS MT",
  "NC ND NE NH NJ NM NV NY OH OK OR PA RI SC SD TN TX UT VA VT WA WI WV WY")," ")[[1]]
stopifnot(nrow(d)==48L, identical(d$state,expected_states))
y <- log(d$packs); w <- log(d$income/d$population/d$cpi); x <- log(d$price/d$cpi)
Z <- cbind(1,w,(d$taxs-d$tax)/d$cpi,d$tax/d$cpi); X <- cbind(1,w,x)
stopifnot(all(is.finite(c(y,X,Z))))
Xhat <- lm.fit(Z,X)$fitted.values
beta <- lm.fit(Xhat,y)$coefficients
u <- drop(y-X%*%beta)
Q <- svd(lm.fit(Xhat,Z)$residuals)$u[,1,drop=FALSE]
score <- Q*u
fit <- lm.fit(score,rep(1,nrow(d)))
statistic <- sum(fit$fitted.values^2)
total <- colSums(score)
direct <- drop(crossprod(total,solve(crossprod(score),total)))
stopifnot(abs(statistic-direct)<1e-12)
print(list(version=R.version.string,N=nrow(d),L=ncol(Z),K=ncol(X),q=1L,
  correction=1,statistic=statistic,p_value=pchisq(statistic,1,lower.tail=FALSE)),digits=16)
expected <- data.frame(state=d$state,lpacks=y,lrprice=x,lrincome=w,tdiff=Z[,3],rtax=Z[,4])
if(length(args)==2) write.csv(expected,args[2],row.names=FALSE)
