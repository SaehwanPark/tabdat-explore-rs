# Development-only Gaussian lm ML oracle for the active zero-unit-variance face.
# Rscript random_effects_ml_boundary_reference.R output.csv
options(digits=17)
library(nlme)
args <- commandArgs(trailingOnly=TRUE); stopifnot(length(args)==1)
rows <- list()
emit <- function(case,path,value) rows[[length(rows)+1L]] <<- data.frame(case,path,value=unname(value))
for(case in c('pooled','shifted_interleaved','both_boundary')) {
  d <- expand.grid(time=0:3,id=1:8)
  d$x <- .3*d$id+.2*d$time+.07*((d$id*d$time+2)%%5)
  d$z <- .13*d$time^2+.03*((d$id+2*d$time)%%4)
  d$y <- (if(case=='both_boundary') 0 else .7*d$x)+c(-.2,.2,.2,-.2)
  if(case=='shifted_interleaved') { d$y <- d$y+100; d <- d[order(-d$time,-d$id),] }
  fit <- lm(y~x+z,data=d); X <- model.matrix(fit); y <- d$y
  N <- nrow(d); K <- ncol(X); G <- 8; T <- 4
  b <- unname(coef(fit)); e <- unname(residuals(fit)); a <- sum(e^2)/N
  Vb <- unname(vcov(fit))*df.residual(fit)/N
  V <- matrix(0,K+1,K+1); V[1:K,1:K] <- Vb; V[K+1,K+1] <- 2/N
  ll <- as.numeric(logLik(fit))
  nll <- function(par) .5*(N*(log(2*pi)+par[K+1])+sum((y-X%*%par[1:K])^2)*exp(-par[K+1]))
  gradient <- function(par) {
    residual <- drop(y-X%*%par[1:K]); variance <- exp(par[K+1])
    c(drop(-crossprod(X,residual)/variance),.5*(N-sum(residual^2)/variance))
  }
  numeric <- optimHess(c(b,log(a)),nll,gradient,control=list(ndeps=rep(1e-5,K+1)))
  H <- solve(V)
  stopifnot(max(abs(numeric-H)/pmax(1,abs(H)))<1e-6)
  Xm <- apply(X,2,function(x)ave(x,d$id)); ym <- ave(y,d$id)
  Xw <- X-Xm; yw <- y-ym; A <- crossprod(Xw); B <- crossprod(Xm)
  c <- crossprod(Xw,yw); f <- crossprod(Xm,ym)
  profile <- function(eta) {
    q <- exp(-eta); beta <- solve(A+q*B,c+q*f)
    sw <- sum((yw-Xw%*%beta)^2); sb <- sum((ym-Xm%*%beta)^2)
    -.5*(N*(log(2*pi)+1+log((sw+q*sb)/N))+G*eta)
  }
  stopifnot(max(sapply(seq(0,24,.1),profile))<=ll+1e-9)
  score <- .5*(sum(ave(e,d$id)^2)/a-G)
  stopifnot(score<0)
  nulla <- sum((y-ym)^2)/(N-G); nullv <- sum((ym-mean(y))^2)/G
  nullboundary <- nullv<=nulla
  if(nullboundary) {
    nulla <- sum((y-mean(y))^2)/N; nullv <- nulla
    nullll <- as.numeric(logLik(lm(y~1,data=d)))
  } else {
    nullll <- -.5*((N-G)*(log(2*pi)+1+log(nulla))+G*(log(2*pi)+1+log(nullv)))
    nullfit <- lme(y~1,random=~1|id,data=d,method='ML',control=lmeControl(tolerance=1e-10,msTol=1e-10))
    stopifnot(abs(as.numeric(logLik(nullfit))-nullll)<1e-8)
  }
  wald <- drop(t(b[-1])%*%solve(Vb[-1,-1,drop=FALSE],b[-1]))
  for(i in 1:K) {
    se <- sqrt(Vb[i,i]); z <- b[i]/se
    fields <- c(value=b[i],standard_error=se,statistic=z,p_value=2*pnorm(-abs(z)))
    for(name in names(fields)) emit(case,paste0('/gls/coefficients/',i-1,'/',name),fields[[name]])
    emit(case,paste0('/confidence_intervals/',i-1,'/0'),b[i]-qnorm(.975)*se)
    emit(case,paste0('/confidence_intervals/',i-1,'/1'),b[i]+qnorm(.975)*se)
    for(j in 1:K) emit(case,paste0('/gls/covariance/matrix/',i-1,'/',j-1),Vb[i,j])
  }
  for(i in 1:(K+1)) for(j in 1:(K+1)) emit(case,paste0('/ml/joint_covariance/',i-1,'/',j-1),V[i,j])
  for(i in 1:N) {
    emit(case,paste0('/gls/fitted_values/',i-1),fitted(fit)[i])
    emit(case,paste0('/gls/residuals/',i-1),e[i])
  }
  vals <- c(sigma_e_squared=a,sigma_u_squared=0,raw_sigma_u_squared=0,theta=0,rho=0,group_count=G,group_size=T,wald_statistic=wald,wald_p_value=pchisq(wald,K-1,lower.tail=FALSE))
  for(name in names(vals)) emit(case,paste0('/',name),vals[[name]])
  vals <- c(eta=0,profile_score=score,log_likelihood=ll,null_log_likelihood=nullll,null_sigma_e_squared=nulla,null_sigma_u_squared=(nullv-nulla)/T,likelihood_ratio=2*(ll-nullll),likelihood_ratio_df=K-1)
  for(name in names(vals)) emit(case,paste0('/ml/',name),vals[[name]])
}
write.csv(do.call(rbind,rows),args[1],row.names=FALSE,quote=FALSE)
cat(R.version.string,'; nlme',as.character(packageVersion('nlme')),';',length(rows),'scalar comparisons\n')
