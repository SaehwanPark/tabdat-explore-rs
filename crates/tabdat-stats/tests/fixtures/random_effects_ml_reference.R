# Development-only interior ML oracle: R/nlme points + independent joint Hessian.
# Rscript random_effects_ml_reference.R output.csv
options(digits=17)
library(nlme)
args <- commandArgs(trailingOnly=TRUE); stopifnot(length(args)==1)
rows <- list()
emit <- function(case,path,value) rows[[length(rows)+1L]] <<- data.frame(case,path,value=unname(value))
for(case in c('balanced','interleaved','shifted')) {
  d <- expand.grid(time=0:3,id=1:8)
  d$x <- .3*d$id+.2*d$time+.07*((d$id*d$time+2)%%5)
  d$z <- .13*d$time^2+.03*((d$id+2*d$time)%%4)
  d$y <- 1+.7*d$x-.3*d$z+.3*((d$id^2+1)%%5)+.11*((d$id*d$time+3)%%7)
  if(case=='shifted') d$y <- d$y+100
  if(case=='interleaved') d <- d[order(-d$time,-d$id),]
  X <- model.matrix(~x+z,d); y <- d$y; G <- 8; T <- 4; N <- nrow(d); K <- ncol(X)
  Xm <- apply(X,2,function(x)ave(x,d$id)); ym <- ave(y,d$id)
  Xw <- X-Xm; yw <- y-ym; A <- crossprod(Xw); B <- crossprod(Xm)
  c <- crossprod(Xw,yw); f <- crossprod(Xm,ym)
  profile <- function(eta) {
    q <- exp(-eta); beta <- unname(drop(solve(A+q*B,c+q*f)))
    ew <- drop(yw-Xw%*%beta); eb <- drop(ym-Xm%*%beta)
    sw <- sum(ew^2); sb <- sum(eb^2); a <- (sw+q*sb)/N; v <- a/q
    list(beta=beta,a=a,v=v,ew=ew,eb=eb,ll=-.5*(N*(log(2*pi)+1+log(a))+G*eta),score=.5*(q*sb/a-G))
  }
  stopifnot(profile(0)$score>0,profile(24)$score<0)
  eta <- uniroot(function(e)profile(e)$score,c(0,24),tol=1e-13)$root
  r <- profile(eta)
  H <- matrix(0,K+2,K+2); H[1:K,1:K] <- A/r$a+B/r$v
  H[1:K,K+1] <- crossprod(Xw,r$ew)/r$a; H[K+1,1:K] <- H[1:K,K+1]
  H[1:K,K+2] <- crossprod(Xm,r$eb)/r$v; H[K+2,1:K] <- H[1:K,K+2]
  H[K+1,K+1] <- sum(r$ew^2)/(2*r$a); H[K+2,K+2] <- sum(r$eb^2)/(2*r$v)
  nll <- function(par) {
    ew <- drop(yw-Xw%*%par[1:K]); eb <- drop(ym-Xm%*%par[1:K])
    .5*((N-G)*par[K+1]+G*par[K+2]+sum(ew^2)*exp(-par[K+1])+sum(eb^2)*exp(-par[K+2])+N*log(2*pi))
  }
  gradient <- function(par) {
    ew <- drop(yw-Xw%*%par[1:K]); eb <- drop(ym-Xm%*%par[1:K]); a <- exp(par[K+1]); v <- exp(par[K+2])
    c(drop(-crossprod(Xw,ew)/a-crossprod(Xm,eb)/v),.5*(N-G-sum(ew^2)/a),.5*(G-sum(eb^2)/v))
  }
  par <- c(r$beta,log(r$a),log(r$v))
  numeric <- optimHess(par,nll,gradient,control=list(ndeps=rep(1e-5,K+2)))
  stopifnot(max(abs(numeric-H)/pmax(1,abs(H)))<1e-6)
  V <- solve(H)
  trusted <- lme(y~x+z,random=~1|id,data=d,method='ML',control=lmeControl(tolerance=1e-10,msTol=1e-10))
  stopifnot(max(abs(fixef(trusted)-r$beta))<1e-6,abs(as.numeric(logLik(trusted))-r$ll)<1e-8)
  grand <- mean(y)
  nullsw <- sum((y-ym)^2); nullsb <- sum((ym-grand)^2)
  nulla <- nullsw/(N-G); nullv <- nullsb/G
  stopifnot(nullv>nulla)
  nullll <- -.5*((N-G)*(log(2*pi)+1+log(nulla))+G*(log(2*pi)+1+log(nullv)))
  trustednull <- lme(y~1,random=~1|id,data=d,method='ML',control=lmeControl(tolerance=1e-10,msTol=1e-10))
  stopifnot(abs(as.numeric(logLik(trustednull))-nullll)<1e-8)
  lr <- 2*(r$ll-nullll)
  emit(case,'/null_log_likelihood',nullll)
  emit(case,'/null_sigma_e_squared',nulla)
  emit(case,'/null_sigma_u_squared',(nullv-nulla)/T)
  emit(case,'/likelihood_ratio',lr)
  emit(case,'/likelihood_ratio_p_value',pchisq(lr,K-1,lower.tail=FALSE))
  for(i in 1:K) {
    se <- sqrt(V[i,i]); z <- r$beta[i]/se
    fields <- c(value=r$beta[i],standard_error=se,statistic=z,p_value=2*pnorm(-abs(z)))
    for(name in names(fields)) emit(case,paste0('/regression/coefficients/',i-1,'/',name),fields[[name]])
  }
  for(i in 1:(K+2)) for(j in 1:(K+2)) emit(case,paste0('/joint_covariance/',i-1,'/',j-1),V[i,j])
  for(i in seq_len(N)) {
    mu <- sum(c(exp(-eta/2),d$x[i]-(1-exp(-eta/2))*Xm[i,2],d$z[i]-(1-exp(-eta/2))*Xm[i,3])*r$beta)
    ys <- d$y[i]-(1-exp(-eta/2))*ym[i]
    emit(case,paste0('/regression/fitted_values/',i-1),mu)
    emit(case,paste0('/regression/residuals/',i-1),ys-mu)
  }
  values <- c(sigma_e_squared=r$a,sigma_u_squared=(r$v-r$a)/T,eta=eta,log_likelihood=r$ll,group_count=G,group_size=T)
  for(name in names(values)) emit(case,paste0('/',name),values[[name]])
}
write.csv(do.call(rbind,rows),args[1],row.names=FALSE,quote=FALSE)
cat(R.version.string,'; nlme',as.character(packageVersion('nlme')),';',length(rows),'scalar comparisons\n')
