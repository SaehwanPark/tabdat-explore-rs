# Development-only unbalanced Gaussian ML oracle; R/nlme plus joint Hessians.
# Rscript random_effects_ml_unbalanced_reference.R reference.csv inputs.csv
options(digits=17)
library(nlme)
args <- commandArgs(trailingOnly=TRUE); stopifnot(length(args)%in%c(2,3))
rows <- list(); inputs <- list()
emit <- function(case,path,value) rows[[length(rows)+1L]] <<- data.frame(case,path,value=unname(value))
for(case in if(length(args)==3) 'mom_sensitivity' else c('unequal','shifted_interleaved','boundary')) {
  sizes <- c(1,2,3,4,5,3,2,4)
  d <- do.call(rbind,lapply(1:8,function(id)data.frame(id,time=seq_len(sizes[id])-1)))
  d$x <- .3*d$id+.2*d$time+.07*((d$id*d$time+2)%%5)
  d$z <- .13*d$time^2+.03*((d$id+2*d$time)%%4)
  d$y <- 1+.7*d$x-.3*d$z+.3*((d$id^2+1)%%5)+.11*((d$id*d$time+3)%%7)
  if(case=='shifted_interleaved') { d$y <- d$y+100; d <- d[order(-d$time,-d$id),] }
  formula <- y~x+z
  if(case=='mom_sensitivity') {
    d <- read.table(args[3],col.names=c('y','x','kids','ageh','agesq','disab','id','time'))
    stopifnot(nrow(d)==5320,identical(as.integer(d$id),rep(1:532,each=10)),identical(as.integer(d$time),rep(1979:1988,532)))
    d$source_row <- seq_len(nrow(d))
    d <- d[!((d$id%%3==0 & d$time%in%c(1980,1985)) | (d$id%%7==0 & d$time>=1981 & d$time<=1983)),]
    stopifnot(nrow(d)==4738,length(unique(d$id))==532)
    formula <- y~x
  }
  X <- model.matrix(formula,d); N <- nrow(d); K <- ncol(X); G <- length(unique(d$id))
  Ti <- ave(d$id,d$id,FUN=length); first <- !duplicated(d$id); Tg <- Ti[first]; ref <- min(Ti)
  Xm <- apply(X,2,function(x)ave(x,d$id)); Xw <- X-Xm
  if(case=='boundary') {
    e <- c(-.2,.2,.2,-.2,.1)[d$time+1]; e <- e-ave(e,d$id)
    e <- e-drop(Xw[,-1]%*%solve(crossprod(Xw[,-1]),crossprod(Xw[,-1],e)))
    d$y <- .7*d$x+e
  }
  y <- d$y; ym <- ave(y,d$id); yw <- y-ym
  if(case!='mom_sensitivity') inputs[[case]] <- cbind(case,d[,c("id","time","y","x","z")])
  make_profile <- function(X) {
    Xm <- apply(X,2,function(x)ave(x,d$id)); Xw <- X-Xm
    function(eta) {
      ratio <- expm1(eta)/ref; q <- 1/(1+Ti*ratio)
      Xt <- Xw+sqrt(q)*Xm; yt <- yw+sqrt(q)*ym
      beta <- unname(drop(solve(crossprod(Xt),crossprod(Xt,yt))))
      ew <- drop(yw-Xw%*%beta); eb <- drop(ym-Xm%*%beta)
      a <- (sum(ew^2)+sum(q*eb^2))/N; u <- a*ratio
      ll <- -.5*(N*(log(2*pi)+1+log(a))+sum(log1p(Tg*ratio)))
      score <- .5*exp(eta)/ref*(sum(eb^2*Ti*q^2)/a-sum(Tg/(1+Tg*ratio)))
      list(beta=beta,ew=ew,eb=eb,a=a,u=u,ll=ll,score=score,Xt=Xt,yt=yt)
    }
  }
  select <- function(profile) {
    grid <- seq(0,24,.25); scores <- sapply(grid,function(eta)profile(eta)$score)
    crossings <- which(head(scores,-1)>0 & tail(scores,-1)<=0)
    roots <- unlist(lapply(crossings,function(i)uniroot(function(e)profile(e)$score,grid[c(i,i+1)],tol=1e-13)$root),use.names=FALSE)
    candidates <- c(0,roots); eta <- candidates[which.max(sapply(candidates,function(e)profile(e)$ll))]
    c(list(eta=eta),profile(eta))
  }
  r <- select(make_profile(X)); null <- select(make_profile(matrix(1,N,1)))
  boundary <- r$eta==0
  stopifnot(boundary==(case=='boundary'))
  if(boundary) {
    trusted <- lm(formula,data=d); Vb <- unname(vcov(trusted))*df.residual(trusted)/N
    V <- matrix(0,K+1,K+1); V[1:K,1:K] <- Vb; V[K+1,K+1] <- 2/N
    nll <- function(par) .5*(N*(log(2*pi)+par[K+1])+sum((y-X%*%par[1:K])^2)*exp(-par[K+1]))
    gradient <- function(par) { e<-drop(y-X%*%par[1:K]); a<-exp(par[K+1]); c(drop(-crossprod(X,e)/a),.5*(N-sum(e^2)/a)) }
    par <- c(r$beta,log(r$a)); H <- solve(V)
    trusted_beta <- unname(coef(trusted))
  } else {
    trusted <- lme(formula,random=list(id=pdDiag(value=log(sqrt(r$u/r$a)),form=~1)),data=d,method='ML',control=lmeControl(tolerance=1e-10,msTol=1e-10))
    trusted_beta <- unname(fixef(trusted))
    v <- r$a+Ti*r$u; wa <- r$a/v; wu <- Ti*r$u/v; sr <- Ti*r$eb^2/v
    H <- matrix(0,K+2,K+2); H[1:K,1:K] <- crossprod(Xw)/r$a+crossprod(Xm,Xm/v)
    H[1:K,K+1] <- crossprod(Xw,r$ew)/r$a+crossprod(Xm,wa*r$eb/v)
    H[1:K,K+2] <- crossprod(Xm,wu*r$eb/v)
    H[K+1,1:K] <- H[1:K,K+1]; H[K+2,1:K] <- H[1:K,K+2]
    H[K+1,K+1] <- sum(r$ew^2)/(2*r$a)+sum(((1-sr)*wa+(-1+2*sr)*wa^2)[first])/2
    H[K+2,K+2] <- sum(((1-sr)*wu+(-1+2*sr)*wu^2)[first])/2
    H[K+1,K+2] <- sum(((-1+2*sr)*wa*wu)[first])/2; H[K+2,K+1] <- H[K+1,K+2]
    V <- solve(H)
    nll <- function(par) {
      a <- exp(par[K+1]); u <- exp(par[K+2]); v <- a+Ti*u
      ew <- drop(yw-Xw%*%par[1:K]); eb <- drop(ym-Xm%*%par[1:K])
      .5*(N*log(2*pi)+(N-G)*log(a)+sum(log(v[first]))+sum(ew^2)/a+sum(eb^2/v))
    }
    gradient <- function(par) {
      a <- exp(par[K+1]); u <- exp(par[K+2]); v <- a+Ti*u
      ew <- drop(yw-Xw%*%par[1:K]); eb <- drop(ym-Xm%*%par[1:K]); sr <- Ti*eb^2/v
      c(drop(-crossprod(Xw,ew)/a-crossprod(Xm,eb/v)),.5*(N-G-sum(ew^2)/a+sum(((1-sr)*a/v)[first])),.5*sum(((1-sr)*Ti*u/v)[first]))
    }
    par <- c(r$beta,log(r$a),log(r$u))
  }
  numeric <- optimHess(par,nll,gradient,control=list(ndeps=rep(1e-5,length(par))))
  stopifnot(max(abs(numeric-H)/pmax(1,abs(H)))<1e-6,all(eigen(H,symmetric=TRUE)$values>0))
  stopifnot(max(abs(trusted_beta-r$beta))<1e-6,abs(as.numeric(logLik(trusted))-r$ll)<1e-8)
  if(null$eta>0) {
    trusted_null <- lme(y~1,random=list(id=pdDiag(value=log(sqrt(null$u/null$a)),form=~1)),data=d,method='ML',control=lmeControl(tolerance=1e-10,msTol=1e-10))
    stopifnot(abs(as.numeric(fixef(trusted_null))-null$beta)<1e-6)
  } else trusted_null <- lm(y~1,data=d)
  stopifnot(abs(as.numeric(logLik(trusted_null))-null$ll)<1e-8)
  for(i in 1:K) {
    se <- sqrt(V[i,i]); z <- r$beta[i]/se
    fields <- c(value=r$beta[i],standard_error=se,statistic=z,p_value=2*pnorm(-abs(z)))
    for(name in names(fields)) emit(case,paste0('/gls/coefficients/',i-1,'/',name),fields[[name]])
    emit(case,paste0('/confidence_intervals/',i-1,'/0'),r$beta[i]-qnorm(.975)*se)
    emit(case,paste0('/confidence_intervals/',i-1,'/1'),r$beta[i]+qnorm(.975)*se)
    for(j in 1:K) emit(case,paste0('/gls/covariance/matrix/',i-1,'/',j-1),V[i,j])
  }
  for(i in seq_len(nrow(V))) for(j in seq_len(ncol(V))) emit(case,paste0('/ml/joint_covariance/',i-1,'/',j-1),V[i,j])
  for(i in 1:N) {
    emit(case,paste0('/gls/fitted_values/',i-1),sum(r$Xt[i,]*r$beta))
    emit(case,paste0('/gls/residuals/',i-1),r$yt[i]-sum(r$Xt[i,]*r$beta))
    emit(case,paste0('/population_fitted_values/',i-1),sum(X[i,]*r$beta))
    emit(case,paste0('/population_residuals/',i-1),y[i]-sum(X[i,]*r$beta))
  }
  if(case=='mom_sensitivity') {
    inputs[[case]] <- data.frame(source_row=d$source_row,lnhr=d$y,lnwg=d$x,kids=d$kids,ageh=d$ageh,agesq=d$agesq,disab=d$disab,id=d$id,year=d$time,ub_xb=drop(X%*%r$beta),ub_residual=drop(y-X%*%r$beta))
    stopifnot(min(Ti)==5,max(Ti)==10)
  }
  # Use the beta principal block explicitly (boundary dimension differs).
  wald <- drop(t(r$beta[-1])%*%solve(V[2:K,2:K,drop=FALSE],r$beta[-1]))
  vals <- c(sigma_e_squared=r$a,sigma_u_squared=r$u,raw_sigma_u_squared=r$u,theta=1-exp(-r$eta/2),rho=r$u/(r$a+r$u),group_count=G,group_size=0,wald_statistic=wald,wald_p_value=pchisq(wald,K-1,lower.tail=FALSE),minimum_group_size=min(Ti),maximum_group_size=max(Ti),theta_min=1-sqrt(r$a/(r$a+min(Ti)*r$u)),theta_max=1-sqrt(r$a/(r$a+max(Ti)*r$u)))
  for(name in names(vals)) emit(case,paste0('/',name),vals[[name]])
  vals <- c(eta=r$eta,profile_score=r$score,log_likelihood=r$ll,null_log_likelihood=null$ll,null_sigma_e_squared=null$a,null_sigma_u_squared=null$u,likelihood_ratio=2*(r$ll-null$ll),likelihood_ratio_df=K-1)
  for(name in names(vals)) emit(case,paste0('/ml/',name),vals[[name]])
  if(!boundary && null$eta>0) emit(case,'/ml/likelihood_ratio_p_value',pchisq(2*(r$ll-null$ll),K-1,lower.tail=FALSE))
}
write.csv(do.call(rbind,rows),args[1],row.names=FALSE,quote=FALSE)
write.csv(do.call(rbind,inputs),args[2],row.names=FALSE,quote=FALSE)
cat(R.version.string,'; nlme',as.character(packageVersion('nlme')),';',length(rows),'scalar comparisons\n')
