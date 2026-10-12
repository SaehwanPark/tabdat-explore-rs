d <- read.csv(commandArgs(trailingOnly=TRUE)[1]);X <- model.matrix(~x1+x2,d);Z <- model.matrix(~x1+z,d);s <- d$selected==1
first <- glm(selected~x1+z,data=d,family=binomial(link='probit'),control=glm.control(epsilon=1e-12,maxit=100));g <- coef(first);eta <- as.vector(Z%*%g)
q <- 2*d$selected-1;zz <- q*eta;mm <- exp(dnorm(zz,log=TRUE)-pnorm(zz,log.p=TRUE));weight <- mm*(mm+zz);Vp <- solve(crossprod(Z,Z*weight))
lam <- exp(dnorm(eta,log=TRUE)-pnorm(eta,log.p=TRUE));W <- cbind(X[s,],lam[s]);b <- qr.coef(qr(W),d$y[s]);u <- d$y[s]-W%*%b;delta <- lam[s]*(lam[s]+eta[s]);sigma2 <- mean(u^2)+tail(b,1)^2*mean(delta);rho <- tail(b,1)/sqrt(sigma2);stopifnot(abs(rho)<1)
C <- solve(crossprod(W));B <- crossprod(W,Z[s,]*delta);S <- sigma2*C%*%(crossprod(W)-rho^2*(crossprod(W,W*delta)-B%*%Vp%*%t(B)))%*%C
values <- c(b,g,sqrt(diag(S)),sqrt(diag(Vp)),as.vector(t(S)),as.vector(t(Vp)),sqrt(sigma2),rho)
cat('R,',paste(sprintf('%.17g',values),collapse=','),'\n',sep='')
