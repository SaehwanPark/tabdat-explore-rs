# R 4.6.1 lm.fit/pf, independent nested OLS; no packages.
i <- 1:40
b <- function(k) cos(2*pi*k*i/40)
w <- b(8); z1 <- b(1); z2 <- b(2); z3 <- b(3)
v1 <- b(4); v2 <- b(5); x1 <- z1+v1; x2 <- z2+v2
# m, include-control, include-intercept, overidentified
for (case in list(c(2,1,1,0),c(2,1,0,0),c(1,0,0,0),c(1,1,1,1),c(2,0,1,1))) {
  m <- case[1]; control <- case[2]==1; intercept <- case[3]==1; overid <- case[4]==1
  y <- 1+0.5*w+1.5*x1+0.6*v1+b(6)
  if (m==2) y <- y-0.7*x2-0.4*v2
  W <- matrix(numeric(0),40,0); if(control) W <- cbind(W,w)
  E <- cbind(x1); if(m==2) E <- cbind(E,x2)
  Z <- cbind(W,z1); if(m==2) Z <- cbind(Z,z2); if(overid) Z <- cbind(Z,z3)
  X <- cbind(W,E)
  if(intercept) { X<-cbind(1,X); Z<-cbind(1,Z) }
  V <- lm.fit(Z,E)$residuals
  restricted <- lm.fit(X,y); augmented <- lm.fit(cbind(X,V),y)
  df <- 40-ncol(X)-m
  F <- (sum(restricted$residuals^2)-sum(augmented$residuals^2))/m/(sum(augmented$residuals^2)/df)
  cat(sprintf("m=%d control=%s intercept=%s overid=%s: F(%d,%d)=%.16g p=%.16g\n",
    m,control,intercept,overid,m,df,F,pf(F,m,df,lower.tail=FALSE)))
}
