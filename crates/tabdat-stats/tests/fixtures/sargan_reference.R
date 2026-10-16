# R 4.6.1, independent QR fits, fixed orthogonal design; no packages needed.
i <- 1:40
b <- function(k) cos(2*pi*k*i/40)
w <- b(8); z1 <- b(1); z2 <- b(2); z3 <- b(3); z4 <- b(7)
x1 <- z1+b(4); x2 <- z2+b(5)
for (extra in c(FALSE, TRUE)) for (intercept in c(TRUE, FALSE)) {
  y <- 1+0.5*w+1.5*x1-0.7*x2+0.3*z3+b(6)+if(extra) 0.2*z4 else 0
  X <- cbind(w,x1,x2); Z <- cbind(w,z1,z2,z3)
  if (extra) Z <- cbind(Z,z4)
  if (intercept) { X <- cbind(1,X); Z <- cbind(1,Z) }
  beta <- lm.fit(lm.fit(Z,X)$fitted.values,y)$coefficients
  u <- y - drop(X %*% beta)
  J <- 40*sum(lm.fit(Z,u)$fitted.values^2)/sum(u^2)
  cat(sprintf("extra %s intercept %s Sargan %.16g df %d p %.16g\n", extra, intercept, J,
    ncol(Z)-ncol(X),pchisq(J,ncol(Z)-ncol(X),lower.tail=FALSE)))
}
