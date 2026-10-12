# Independent textbook 2SLS using QR projection; no naive second-stage residuals.
a <- read.csv(commandArgs(trailingOnly=TRUE)[1]);stopifnot(nrow(a)==160)
x <- cbind(1,a$x,a$d);z <- cbind(1,a$x,a$z1,a$z2);y <- a$y
stopifnot(qr(x)$rank==3,qr(z)$rank==4)
xhat <- qr.fitted(qr(z),x)
b <- solve(crossprod(xhat,x),crossprod(xhat,y))
u <- as.vector(y-x%*%b)
covariance <- sum(u^2)/length(y)*solve(crossprod(xhat,x))
values <- c(as.vector(b),sqrt(diag(covariance)),as.vector(t(covariance)),1-sum(u^2)/sum((y-mean(y))^2))
cat("R,iv,",paste(sprintf("%.17g",values),collapse=","),"\n",sep="")
cat(R.version.string,"\n",file=stderr())
