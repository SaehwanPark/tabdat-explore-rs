# Development-only independent Gaussian profile ML / observed-information oracle.
# Rscript household_re_ml_reference.R source fresh-directory [--dta|--csv]
options(digits=17)
a <- commandArgs(trailingOnly=TRUE)
if (length(a)<2 || length(a)>3 || (length(a)==3 && !a[3]%in%c('--dta','--csv'))) stop('usage: household_re_ml_reference.R source fresh-directory [--dta|--csv]')
if (file.exists(a[2])) stop('reference directory already exists')
if (length(a)==3 && a[3]=='--csv') {
  source <- read.csv(a[1])
} else if (length(a)==3 && a[3]=='--dta') {
  raw <- foreign::read.dta(a[1],convert.factors=FALSE)
  source <- data.frame(source_row=seq_len(nrow(raw)),lnexp12m=raw$lhhex12m,age=raw$age,sex=raw$sex,hhsize=raw$hhsize,farm=raw$farm,educ=raw$comped98,lnhhexp=raw$lhhexp1,commune=raw$commune)
} else {
  tokens <- scan(a[1],what=character(),quote='"',quiet=TRUE)
  stopifnot(length(tokens)==5999*8)
  raw <- matrix(tokens,ncol=8,byrow=TRUE)
  farms <- c('non farm'=0,'farm'=1)
  education <- c('Never'=0,'Nhatre'=1,'<cap I'=2,'Cap I'=3,'Cap II'=4,'Cap III'=5,'Nghe SC'=6,'THCN'=7,'DHDC'=8,'DHCD'=9,'Thac sy'=10,'PTS'=11,'TS'=12)
  num <- function(x) as.numeric(replace(x,x=='.',NA))
  source <- data.frame(source_row=seq_len(nrow(raw)),lnexp12m=num(raw[,1]),age=num(raw[,2]),sex=num(raw[,3]),hhsize=num(raw[,4]),farm=unname(farms[raw[,5]]),educ=unname(education[raw[,6]]),lnhhexp=num(raw[,7]),commune=num(raw[,8]))
}
stopifnot(!anyDuplicated(source$source_row),!anyNA(source[,-2]))
d <- source[!is.na(source$lnexp12m),]; y <- d$lnexp12m
X <- model.matrix(lnexp12m~lnhhexp+age+sex+hhsize+farm+educ,d)
N <- nrow(d); K <- ncol(X); G <- length(unique(d$commune)); Ti <- ave(d$commune,d$commune,FUN=length)
first <- !duplicated(d$commune); Tg <- Ti[first]; ref <- min(Ti)
ym <- ave(y,d$commune); yw <- y-ym
make_profile <- function(X) {
  Xm <- apply(X,2,function(x)ave(x,d$commune)); Xw <- X-Xm
  function(eta) {
    ratio <- expm1(eta)/ref; q <- 1/(1+Ti*ratio)
    Xt <- Xw+sqrt(q)*Xm; yt <- yw+sqrt(q)*ym
    fit <- lm.fit(Xt,yt); stopifnot(fit$rank==ncol(Xt))
    beta <- unname(fit$coefficients);ew <- drop(yw-Xw%*%beta);eb <- drop(ym-Xm%*%beta)
    e <- (sum(ew^2)+sum(q*eb^2))/N; u <- e*ratio
    ll <- -.5*(N*(log(2*pi)+1+log(e))+sum(log1p(Tg*ratio)))
    score <- .5*exp(eta)/ref*(sum(eb^2*Ti*q^2)/e-sum(Tg/(1+Tg*ratio)))
    list(beta=beta,ew=ew,eb=eb,e=e,u=u,ll=ll,score=score,Xt=Xt,yt=yt,Xm=Xm,Xw=Xw)
  }
}
select <- function(profile) {
  grid <- seq(0,24,.25); scores <- sapply(grid,function(eta)profile(eta)$score)
  crossings <- which(head(scores,-1)>0 & tail(scores,-1)<=0)
  roots <- unlist(lapply(crossings,function(i)uniroot(function(e)profile(e)$score,grid[c(i,i+1)],tol=1e-13)$root),use.names=FALSE)
  candidates <- c(0,roots);eta <- candidates[which.max(sapply(candidates,function(e)profile(e)$ll))]
  c(list(eta=eta),profile(eta))
}
profile <- make_profile(X); r <- select(profile); null <- select(make_profile(matrix(1,N,1)))
boundary <- r$eta==0
if (!boundary) {
  trusted <- nlme::lme(lnexp12m~lnhhexp+age+sex+hhsize+farm+educ,random=~1|commune,data=d,method='ML',control=nlme::lmeControl(tolerance=1e-10,msTol=1e-10,msMaxIter=200))
  trusted_beta <- unname(nlme::fixef(trusted))
} else {
  trusted <- lm(lnexp12m~lnhhexp+age+sex+hhsize+farm+educ,data=d)
  trusted_beta <- unname(coef(trusted))
}
trusted_null <- if (null$eta>0) nlme::lme(lnexp12m~1,random=~1|commune,data=d,method='ML',control=nlme::lmeControl(tolerance=1e-10,msTol=1e-10,msMaxIter=200)) else lm(lnexp12m~1,data=d)
stopifnot(max(abs(trusted_beta-r$beta))<1e-6,abs(as.numeric(logLik(trusted))-r$ll)<1e-8,abs(as.numeric(logLik(trusted_null))-null$ll)<1e-8)
if (!boundary) {
  v <- r$e+Ti*r$u; wa <- r$e/v; wu <- Ti*r$u/v; sr <- Ti*r$eb^2/v
  H <- matrix(0,K+2,K+2);H[1:K,1:K] <- crossprod(r$Xw)/r$e+crossprod(r$Xm,r$Xm/v)
  H[1:K,K+1] <- crossprod(r$Xw,r$ew)/r$e+crossprod(r$Xm,wa*r$eb/v)
  H[1:K,K+2] <- crossprod(r$Xm,wu*r$eb/v)
  H[K+1,1:K] <- H[1:K,K+1];H[K+2,1:K] <- H[1:K,K+2]
  H[K+1,K+1] <- sum(r$ew^2)/(2*r$e)+sum(((1-sr)*wa+(-1+2*sr)*wa^2)[first])/2
  H[K+2,K+2] <- sum(((1-sr)*wu+(-1+2*sr)*wu^2)[first])/2
  H[K+1,K+2] <- sum(((-1+2*sr)*wa*wu)[first])/2;H[K+2,K+1] <- H[K+1,K+2]
  V <- solve(H)
  nll <- function(par) {
    e <- exp(par[K+1]);u <- exp(par[K+2]);v <- e+Ti*u
    ew <- drop(yw-r$Xw%*%par[1:K]);eb <- drop(ym-r$Xm%*%par[1:K])
    .5*(N*log(2*pi)+(N-G)*log(e)+sum(log(v[first]))+sum(ew^2)/e+sum(eb^2/v))
  }
  gradient <- function(par) {
    e <- exp(par[K+1]);u <- exp(par[K+2]);v <- e+Ti*u
    ew <- drop(yw-r$Xw%*%par[1:K]);eb <- drop(ym-r$Xm%*%par[1:K]);sr <- Ti*eb^2/v
    c(drop(-crossprod(r$Xw,ew)/e-crossprod(r$Xm,eb/v)),.5*(N-G-sum(ew^2)/e+sum(((1-sr)*e/v)[first])),.5*sum(((1-sr)*Ti*u/v)[first]))
  }
  par <- c(r$beta,log(r$e),log(r$u))
} else {
  V <- matrix(0,K+1,K+1);V[1:K,1:K] <- r$e*chol2inv(qr.R(qr(X)));V[K+1,K+1] <- 2/N;H <- solve(V)
  nll <- function(par) .5*(N*(log(2*pi)+par[K+1])+sum((y-X%*%par[1:K])^2)*exp(-par[K+1]))
  gradient <- function(par) {e <- exp(par[K+1]);res <- drop(y-X%*%par[1:K]);c(drop(-crossprod(X,res)/e),.5*(N-sum(res^2)/e))}
  par <- c(r$beta,log(r$e))
}
numeric <- optimHess(par,nll,gradient,control=list(ndeps=rep(1e-5,length(par))))
stopifnot(max(abs(numeric-H)/pmax(1,abs(H)))<1e-6,all(eigen(H,symmetric=TRUE)$values>0))
refs <- list(); emit <- function(path,value) refs[[length(refs)+1L]] <<- data.frame(path,value=unname(value))
for (i in 1:K) {
  se <- sqrt(V[i,i]);z <- r$beta[i]/se
  for (name in c('value','standard_error','statistic','p_value','ci_lower','ci_upper')) emit(sprintf('/coefficients/%d/%s',i-1,name),switch(name,value=r$beta[i],standard_error=se,statistic=z,p_value=2*pnorm(-abs(z)),ci_lower=r$beta[i]-qnorm(.975)*se,ci_upper=r$beta[i]+qnorm(.975)*se))
  for (j in 1:K) emit(sprintf('/covariance_matrix/%d/%d',i-1,j-1),V[i,j])
}
for (i in seq_len(nrow(V))) for (j in seq_len(ncol(V))) emit(sprintf('/ml/joint_covariance/%d/%d',i-1,j-1),V[i,j])
for (i in seq_len(if (boundary) 1 else 2)) {
  name <- c('sigma_e_squared','sigma_u_squared')[i]; est <- c(r$e,r$u)[i];lse <- sqrt(V[K+i,K+i]); ci <- exp(log(est)+c(-1,1)*qnorm(.975)*lse)
  for (field in c('estimate','standard_error','log_scale_standard_error')) emit(paste0('/ml/variance_component_inference/',name,'/',field),switch(field,estimate=est,standard_error=est*lse,log_scale_standard_error=lse))
  for (j in 1:2) emit(sprintf('/ml/variance_component_inference/%s/confidence_interval/%d',name,j-1),ci[j])
}
w <- drop(t(r$beta[-1])%*%solve(V[2:K,2:K],r$beta[-1]));lr <- 2*(r$ll-null$ll);unit_lr <- max(0,2*(r$ll-profile(0)$ll));rss <- sum((r$yt-r$Xt%*%r$beta)^2)
values <- c(observation_count=N,group_count=G,minimum_group_size=min(Tg),maximum_group_size=max(Tg),df_residual=N-K,sigma_e_squared=r$e,sigma_u_squared=r$u,raw_sigma_u_squared=r$u,rho=r$u/(r$e+r$u),theta_min=1-sqrt(r$e/(r$e+min(Tg)*r$u)),theta_max=1-sqrt(r$e/(r$e+max(Tg)*r$u)),theta_at_minimum_group_size=1-sqrt(r$e/(r$e+min(Tg)*r$u)),transformed_rss=rss,transformed_root_mse=sqrt(rss/(N-K)),wald_statistic=w,wald_df=K-1,wald_p_value=pchisq(w,K-1,lower.tail=FALSE))
for(name in names(values)) emit(paste0('/',name),values[[name]])
values <- c(eta=r$eta,profile_score=r$score,log_likelihood=r$ll,null_log_likelihood=null$ll,null_sigma_e_squared=null$e,null_sigma_u_squared=null$u,likelihood_ratio=lr,likelihood_ratio_df=K-1)
if (!boundary && null$eta>0) values <- c(values,likelihood_ratio_p_value=pchisq(lr,K-1,lower.tail=FALSE))
for(name in names(values)) emit(paste0('/ml/',name),values[[name]])
emit('/ml/unit_variance_test/statistic',unit_lr);emit('/ml/unit_variance_test/p_value',if (unit_lr==0) 1 else .5*pchisq(unit_lr,1,lower.tail=FALSE))
dir.create(a[2]);write.csv(do.call(rbind,refs),file.path(a[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write_precise <- function(x,path) {
  x[] <- lapply(x,function(v)ifelse(is.na(v),NA_character_,sprintf('%.17g',v)))
  write.csv(x,path,row.names=FALSE,quote=FALSE,na='')
}
write_precise(source,file.path(a[2],'source.csv'))
d$ml_xb <- drop(X%*%r$beta);d$ml_residual <- y-d$ml_xb
write_precise(d,file.path(a[2],'predictions.csv'))
cat(R.version.string,'; nlme',as.character(packageVersion('nlme')),';',N,'observations;',G,'communes;',length(refs),'quantities\n')
