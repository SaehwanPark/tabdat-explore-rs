# Development-only base-R independent SVD/lm.fit Wooldridge overid score.
# Structural residuals are against ORIGINAL X, not the projected second stage.
i <- 1:40
b <- function(k) cos(2*pi*k*i/40)
w <- b(8); z1 <- b(1); z2 <- b(2); z3 <- b(3); z4 <- b(7)
x1 <- 0.7*z1+0.4*z2-0.2*z3+b(4)
x2 <- -0.3*z1+1.2*z2+0.5*z3+b(5)
cat(R.version.string,"\n")
for(case in list(c(2,1,1,0),c(2,1,0,0),c(2,1,1,1),c(2,1,0,1),c(1,0,0,0))) {
  m <- case[1]; control <- case[2]==1; intercept <- case[3]==1; extra <- case[4]==1
  y <- 1+0.5*w+1.5*x1+0.3*z3+(1+0.2*w)*b(6)
  if(m==2) y <- y-0.7*x2
  W <- matrix(numeric(0),40,0); if(control) W <- cbind(W,w)
  X <- cbind(W,x1); if(m==2) X <- cbind(X,x2)
  Z <- cbind(W,z1,z2,z3)
  if(extra) { Z<-cbind(Z,z4); y<-y+0.2*z4 }
  if(intercept) { X<-cbind(1,X); Z<-cbind(1,Z) }
  Xhat <- as.matrix(lm.fit(Z,X)$fitted.values)
  beta <- lm.fit(Xhat,y)$coefficients
  u <- drop(y-X%*%beta)
  residualized <- lm.fit(Xhat,Z)$residuals
  q <- ncol(Z)-ncol(X)
  Q <- svd(residualized)$u[,seq_len(q),drop=FALSE]
  score <- Q*u
  fit <- lm.fit(score,rep(1,40))
  statistic <- sum(fit$fitted.values^2)
  # Independent direct moment quadratic confirms the projection identity.
  total <- colSums(score)
  direct <- drop(crossprod(total,solve(crossprod(score),total)))
  stopifnot(abs(statistic-direct)<1e-12)
  cat(sprintf("m=%d control=%s intercept=%s extra=%s L=%d K=%d q=%d score=%.16g p=%.16g\n",
    m,control,intercept,extra,ncol(Z),ncol(X),q,statistic,pchisq(statistic,q,lower.tail=FALSE)))
}
# CLI-design block: confidence_fixture(2,0,.3), separate orthogonal frequencies.
w <- b(5); x <- 2*z1+b(3); y <- 1.5*x+0.5*w+0.3*z2+b(4)
X <- cbind(1,w,x); Z <- cbind(1,w,z1,z2)
H <- lm.fit(Z,X)$fitted.values
u <- drop(y-X%*%lm.fit(H,y)$coefficients)
Q <- svd(lm.fit(H,Z)$residuals)$u[,1,drop=FALSE]
value <- sum(lm.fit(Q*u,rep(1,40))$fitted.values^2)
cat(sprintf("CLI q=1 score=%.16g p=%.16g\n",value,pchisq(value,1,lower.tail=FALSE)))
