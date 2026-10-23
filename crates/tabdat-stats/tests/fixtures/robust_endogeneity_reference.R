# Development-only base-R residual-inclusion Wald reference; no packages.
# Same five deterministic designs as wu_hausman_reference.R / tests/wu_hausman.rs.
i <- 1:40
b <- function(k) cos(2*pi*k*i/40)
w <- b(8); z1 <- b(1); z2 <- b(2); z3 <- b(3)
v1 <- b(4); v2 <- b(5); x1 <- z1+v1; x2 <- z2+v2
group <- (i-1) %/% 4
cat(R.version.string, "\n")
for (case in list(c(2,1,1,0),c(2,1,0,0),c(1,0,0,0),c(1,1,1,1),c(2,0,1,1))) {
  m <- case[1]; control <- case[2]==1; intercept <- case[3]==1; overid <- case[4]==1
  y <- 1+0.5*w+1.5*x1+0.6*v1+b(6)
  if(m==2) y <- y-0.7*x2-0.4*v2
  W <- matrix(numeric(0),40,0); if(control) W <- cbind(W,w)
  E <- cbind(x1); if(m==2) E <- cbind(E,x2)
  Z <- cbind(W,z1); if(m==2) Z <- cbind(Z,z2); if(overid) Z <- cbind(Z,z3)
  X <- cbind(W,E)
  if(intercept) { X<-cbind(1,X); Z<-cbind(1,Z) }
  V <- lm.fit(Z,E)$residuals
  A <- cbind(X,V); fit <- lm.fit(A,y)
  N <- nrow(A); K <- ncol(A); G <- length(unique(group)); j <- (K-m+1):K
  bread <- solve(crossprod(A)); scores <- A*drop(fit$residuals)
  for (mode in c("hc1","cluster")) {
    correction <- if(mode=="hc1") N/(N-K) else G/(G-1)*(N-1)/(N-K)
    S <- if(mode=="hc1") scores else rowsum(scores,group)
    covariance <- correction * bread %*% crossprod(S) %*% bread
    value <- fit$coefficients[j]
    statistic <- drop(crossprod(value,solve(covariance[j,j,drop=FALSE],value)))
    cat(sprintf("m=%d control=%s intercept=%s overid=%s %s: K=%d df=%d G=%d correction=%.16g W=%.16g p=%.16g\n",
      m,control,intercept,overid,mode,K,N-K,G,correction,statistic,
      pchisq(statistic,m,lower.tail=FALSE)))
  }
}
