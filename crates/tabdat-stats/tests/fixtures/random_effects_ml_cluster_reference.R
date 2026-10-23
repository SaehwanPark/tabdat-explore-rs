# Development-only: Rscript <file> output-directory [original-MOM.dat].
# Dense per-unit marginal Gaussian derivatives, joint observed bread, G/(G-1).
options(digits=17)
library(nlme)
args <- commandArgs(TRUE); stopifnot(length(args)%in%1:2)
out <- args[1]; dir.create(out,recursive=TRUE,showWarnings=FALSE)
refs <- list(); inputs <- list()
emit <- function(case,path,value) refs[[length(refs)+1L]] <<- data.frame(case,path,value=unname(value))
cases <- if(length(args)==2) 'mom' else c('balanced','unequal','shifted_interleaved','null_boundary')
for(case in cases) {
  sizes <- if(case%in%c('unequal','shifted_interleaved')) c(1,2,3,4,5,3,2,4) else rep(4,8)
  d <- do.call(rbind,lapply(1:8,function(id)data.frame(id,time=seq_len(sizes[id])-1)))
  d$x <- .3*d$id+.2*d$time+.07*((d$id*d$time+2)%%5)
  d$z <- .13*d$time^2+.03*((d$id+2*d$time)%%4)
  d$y <- 1+.7*d$x-.3*d$z+.3*((d$id^2+1)%%5)+.11*((d$id*d$time+3)%%7)
  if(case=='shifted_interleaved') {d$y<-d$y+100; d<-d[order(-d$time,-d$id),]}
  if(case=='null_boundary') d$y <- .7*(d$x-ave(d$x,d$id))+.002*c(-1,1,1,-1)
  formula <- y~x+z
  if(case=='mom') {
    d <- read.table(args[2],col.names=c('y','x','kids','ageh','agesq','disab','id','time'))
    stopifnot(nrow(d)==5320,identical(as.integer(d$id),rep(1:532,each=10)),identical(as.integer(d$time),rep(1979:1988,532)))
    formula <- y~x
  }
  d$source_row <- seq_len(nrow(d))
  if(case!='mom') {
    # Raw keys are valid, but this unit is omitted by joint finite-case selection.
    sentinel <- d[1,,drop=FALSE]; sentinel$id<-99; sentinel$time<-0; sentinel$y<-NA; sentinel$source_row<-nrow(d)+1L
    inputs[[case]] <- cbind(case,rbind(d,sentinel)[,c('source_row','id','time','y','x','z')])
  }
  X <- model.matrix(formula,d); y <- d$y; N<-length(y); K<-ncol(X); ids<-sort(unique(d$id)); G<-length(ids)
  ix <- lapply(ids,function(id)which(d$id==id)); Ti<-ave(d$id,d$id,FUN=length); first<-!duplicated(d$id); Tg<-Ti[first]; ref<-min(Ti)
  balanced <- length(unique(Ti))==1
  Xm<-apply(X,2,function(x)ave(x,d$id)); Xw<-X-Xm; ym<-ave(y,d$id); yw<-y-ym
  make_profile <- function(design) {
    means<-apply(design,2,function(x)ave(x,d$id)); within<-design-means
    function(eta) {
      ratio<-expm1(eta)/ref; q<-1/(1+Ti*ratio)
      Xt<-within+sqrt(q)*means; yt<-yw+sqrt(q)*ym
      beta<-drop(solve(crossprod(Xt),crossprod(Xt,yt)))
      ew<-drop(yw-within%*%beta); eb<-drop(ym-means%*%beta)
      a<-(sum(ew^2)+sum(q*eb^2))/N; u<-a*ratio
      ll<- -.5*(N*(log(2*pi)+1+log(a))+sum(log1p(Tg*ratio)))
      score<-.5*exp(eta)/ref*(sum(eb^2*Ti*q^2)/a-sum(Tg/(1+Tg*ratio)))
      list(beta=beta,a=a,u=u,ll=ll,ew=ew,eb=eb,score=score)
    }
  }
  select <- function(profile) {
    grid<-seq(0,24,.25); score<-sapply(grid,function(e)profile(e)$score)
    cross<-which(head(score,-1)>0 & tail(score,-1)<=0)
    roots<-unlist(lapply(cross,function(j)uniroot(function(e)profile(e)$score,grid[c(j,j+1)],tol=1e-13)$root))
    candidates<-c(0,roots); eta<-candidates[which.max(sapply(candidates,function(e)profile(e)$ll))]
    c(list(eta=eta),profile(eta))
  }
  r<-select(make_profile(X)); stopifnot(r$eta>0)
  null<-select(make_profile(matrix(1,N,1)))
  trusted<-lme(formula,random=list(id=pdDiag(value=log(sqrt(r$u/r$a)),form=~1)),data=d,method='ML',control=lmeControl(tolerance=1e-10,msTol=1e-10))
  stopifnot(max(abs(fixef(trusted)-r$beta))<1e-6,abs(as.numeric(logLik(trusted))-r$ll)<1e-8)
  # Full joint observed information in the native coordinate system, separately
  # checked against numerical derivatives of the dense marginal likelihood.
  variance<-r$a+Ti*r$u
  H<-matrix(0,K+2,K+2); H[1:K,1:K]<-crossprod(Xw)/r$a+crossprod(Xm,Xm/variance)
  if(balanced) {
    H[1:K,K+1]<-crossprod(Xw,r$ew)/r$a; H[1:K,K+2]<-crossprod(Xm,r$eb/variance)
    H[K+1,K+1]<-sum(r$ew^2)/(2*r$a); H[K+2,K+2]<-sum(r$eb^2/variance)/2
  } else {
    wa<-r$a/variance; wu<-Ti*r$u/variance; sr<-Ti*r$eb^2/variance
    H[1:K,K+1]<-crossprod(Xw,r$ew)/r$a+crossprod(Xm,wa*r$eb/variance)
    H[1:K,K+2]<-crossprod(Xm,wu*r$eb/variance)
    H[K+1,K+1]<-sum(r$ew^2)/(2*r$a)+sum(((1-sr)*wa+(-1+2*sr)*wa^2)[first])/2
    H[K+2,K+2]<-sum(((1-sr)*wu+(-1+2*sr)*wu^2)[first])/2
    H[K+1,K+2]<-sum(((-1+2*sr)*wa*wu)[first])/2; H[K+2,K+1]<-H[K+1,K+2]
  }
  H[K+1,1:K]<-H[1:K,K+1]; H[K+2,1:K]<-H[1:K,K+2]
  pars<-c(r$beta,log(r$a),log(if(balanced) variance[1] else r$u))
  unit <- function(par,index,derivative=FALSE) {
    n<-length(index); a<-exp(par[K+1]); J<-matrix(1,n,n); I<-diag(n)
    if(balanced) {v<-exp(par[K+2]); V<-a*(I-J/n)+v*J/n; D<-list(a*(I-J/n),v*J/n)}
    else {u<-exp(par[K+2]); V<-a*I+u*J; D<-list(a*I,u*J)}
    e<-drop(y[index]-X[index,,drop=FALSE]%*%par[1:K]); P<-solve(V); Pe<-drop(P%*%e)
    if(!derivative) return(-.5*(n*log(2*pi)+as.numeric(determinant(V,logarithm=TRUE)$modulus)+sum(e*Pe)))
    c(drop(crossprod(X[index,,drop=FALSE],Pe)),sapply(D,function(D).5*(sum(Pe*(D%*%Pe))-sum(diag(P%*%D)))))
  }
  scores<-t(sapply(ix,function(index)unit(pars,index,TRUE)))
  # Per-unit finite differences test all nuisance-coordinate scores, not beta alone.
  numeric_scores<-t(sapply(ix,function(index)sapply(seq_along(pars),function(j) {
    step<-1e-5; hi<-lo<-pars; hi[j]<-hi[j]+step; lo[j]<-lo[j]-step
    (unit(hi,index)-unit(lo,index))/(2*step)
  })))
  stopifnot(max(abs(scores-numeric_scores)/pmax(1,abs(scores)))<2e-6)
  {
    gradient<-function(par)-colSums(t(sapply(ix,function(index)unit(par,index,TRUE))))
    # Richardson steps avoid subtracting nearly equal ill-conditioned dense
    # scores at 1e-5 in the tiny-residual-variance/null-boundary design.
    derivative<-function(j,h) {hi<-lo<-pars; hi[j]<-hi[j]+h; lo[j]<-lo[j]-h; (gradient(hi)-gradient(lo))/(2*h)}
    numeric_H<-sapply(seq_along(pars),function(j)(4*derivative(j,.005)-derivative(j,.01))/3)
    stopifnot(max(abs(H-numeric_H)/pmax(1,abs(H)))<2e-6)
  }
  bread<-solve(H); correction<-G/(G-1); V<-correction*bread%*%crossprod(scores)%*%bread
  for(i in 1:(K+2)) for(j in 1:(K+2)) emit(case,paste0('/ml/joint_covariance/',i-1,'/',j-1),V[i,j])
  critical<-qnorm(.975)
  for(i in 1:K) {
    se<-sqrt(V[i,i]); b<-r$beta[i]
    for(field in c('value','standard_error','statistic','p_value')) {
      value<-switch(field,value=b,standard_error=se,statistic=b/se,p_value=2*pnorm(-abs(b/se)))
      emit(case,paste0('/gls/coefficients/',i-1,'/',field),value)
    }
    emit(case,paste0('/confidence_intervals/',i-1,'/0'),b-critical*se)
    emit(case,paste0('/confidence_intervals/',i-1,'/1'),b+critical*se)
    for(j in 1:K) emit(case,paste0('/gls/covariance/matrix/',i-1,'/',j-1),V[i,j])
  }
  component<-function(name,estimate,variance) {
    se<-sqrt(variance); bounds<-exp(log(estimate)+c(-1,1)*critical*se)
    for(field in c('estimate','standard_error','log_scale_standard_error'))
      emit(case,paste0('/ml/variance_component_inference/',name,'/',field),switch(field,estimate=estimate,standard_error=estimate*se,log_scale_standard_error=se))
    for(j in 1:2) emit(case,paste0('/ml/variance_component_inference/',name,'/confidence_interval/',j-1),bounds[j])
  }
  component('sigma_e_squared',r$a,V[K+1,K+1])
  gu<-if(balanced) c(-r$a/(variance[1]-r$a),variance[1]/(variance[1]-r$a)) else c(0,1)
  component('sigma_u_squared',r$u,drop(t(gu)%*%V[(K+1):(K+2),(K+1):(K+2)]%*%gu))
  W<-drop(t(r$beta[-1])%*%solve(V[2:K,2:K,drop=FALSE],r$beta[-1]))
  values<-c(sigma_e_squared=r$a,sigma_u_squared=r$u,theta=1-exp(-r$eta/2),rho=r$u/(r$a+r$u),group_count=G,wald_statistic=W,wald_p_value=pchisq(W,K-1,lower.tail=FALSE))
  for(name in names(values)) emit(case,paste0('/',name),values[[name]])
  emit(case,'/ml/log_likelihood',r$ll); emit(case,'/ml/null_log_likelihood',null$ll); emit(case,'/ml/likelihood_ratio',2*(r$ll-null$ll))
  emit(case,'/ml/cluster/finite_sample_multiplier',correction)
  prediction<-d; prediction$population_xb<-drop(X%*%r$beta); prediction$population_residual<-y-prediction$population_xb
  prediction$theta<-1-sqrt(r$a/(r$a+Ti*r$u))
  if(case=='mom') names(prediction)[match(c('y','x','time'),names(prediction))]<-c('lnhr','lnwg','year')
  write.csv(prediction,file.path(out,paste0(case,'_predictions.csv')),row.names=FALSE,quote=FALSE)
}
write.csv(do.call(rbind,refs),file.path(out,'reference.csv'),row.names=FALSE,quote=FALSE)
if(length(inputs)) write.table(do.call(rbind,inputs),file.path(out,'inputs.csv'),sep=',',row.names=FALSE,quote=FALSE,na='')
cat(R.version.string,'; nlme',as.character(packageVersion('nlme')),';',length(refs),'scalar comparisons\n')
