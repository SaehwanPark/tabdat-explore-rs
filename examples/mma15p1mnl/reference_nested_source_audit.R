# Development-only source audit; not a native estimator or a finite-MLE claim.
# Args: Nldata.csv, R_nested_quantities.csv, R_normalized_common_quantities.csv.
# R4.6.1/mlogit2.0.0. Stdout is diagnostics CSV; inputs remain read-only.
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==3)
library(mlogit)
stopifnot(packageVersion('mlogit')=='2.0.0')
options(digits=17)
w <- read.csv(args[1]);N <- nrow(w)
stopifnot(N==1182,identical(w$source_row,seq_len(N)),all(is.finite(as.matrix(w))),
  all(w$mode %in% 1:4),all(as.matrix(w[c('dbeach','dpier','dprivate','dcharter')])==outer(w$mode,1:4,'==')))
old <- read.csv(args[2]);common <- read.csv(args[3])
stopifnot(nrow(old)==149,nrow(common)==131)
parameters <- function(data,model) {
  rows <- data[data$model==model & data$quantity=='parameter_value',]
  stopifnot(identical(rows$i,seq_len(nrow(rows))))
  rows$value
}
stationary <- parameters(old,'nested');beta <- parameters(common,'normalized_common')
restricted <- parameters(common,'restricted')
stopifnot(length(stationary)==6,length(beta)==5,length(restricted)==4)
records <- list()
emit <- function(section,quantity,value,i=0,j=0) {
  stopifnot(length(value)==1,is.finite(value))
  records[[length(records)+1L]] <<- data.frame(section=section,quantity=quantity,i=i,j=j,value=value)
}
pnames <- c('pbeach','ppier','pprivate','pcharter');qnames <- c('qbeach','qpier','qprivate','qcharter')
p <- as.matrix(w[pnames]);q <- as.matrix(w[qnames]);y <- w$income/1000
ys <- as.numeric(w$mode<=2);ss <- ifelse(w$mode==1,1,-1);sb <- ifelse(w$mode==3,1,-1)
dps <- p[,1]-p[,2];dqs <- q[,1]-q[,2];dpb <- p[,3]-p[,4];dqb <- q[,3]-q[,4]
dmp <- (p[,1]+p[,2]-p[,3]-p[,4])/2;dmq <- (q[,1]+q[,2]-q[,3]-q[,4])/2
stopifnot(all(dps==0))
emit('sample','decisions',N);emit('sample','long_rows',4*N)
for(i in 1:4)emit('sample','chosen_count',sum(w$mode==i),i)
# Exact stable identities, not a truncated small-argument likelihood series.
logcosh <- function(x) {
  small <- abs(x)<=20;result <- numeric(length(x))
  result[small] <- log1p(2*sinh(x[small]/2)^2)
  z <- abs(x[!small]);result[!small] <- z+log1p(exp(-2*z))-log(2)
  result
}
entropic <- function(k,d) {if(k==0)return(rep(0,length(d)));logcosh(k*d/2)/k}
entropic_derivative <- function(k,d) {
  if(k==0)return(d^2/8)
  x <- k*d/2;small <- abs(x)<=20;result <- numeric(length(x))
  result[small] <- x[small]*tanh(x[small])-logcosh(x[small])
  z <- abs(x[!small]);e <- exp(-2*z)
  result[!small] <- log(2)-log1p(e)-2*z*e/(1+e)
  result/k^2
}
# ks=1/lambda_shore, kb=1/lambda_boat. Subtract log(2) inside each
# inclusive value and absorb (lambda_shore-lambda_boat)*log(2) in a.
# pc=100*beta_price and kb=.05*theta6 are fixed numerical coordinates only.
# The zero reciprocal-scale closure has uniform within-nest probabilities;
# it is NOT a finite positive-scale fit in the original parameterization.
evaluate <- function(theta) {
  bp <- theta[1]/100;bq <- theta[2];a <- theta[3];b <- theta[4]
  ks <- theta[5];kb <- .05*theta[6]
  ds <- bp*dps+bq*dqs;db <- bp*dpb+bq*dqb
  ts <- tanh(ks*ds/2);tb <- tanh(kb*db/2)
  eta <- a+b*y+bp*dmp+bq*dmq+entropic(ks,ds)-entropic(kb,db)
  S <- plogis(eta);res <- ys-S
  within <- ys*(ss*ks*ds/2-logcosh(ks*ds/2)-log(2))+
    (1-ys)*(sb*kb*db/2-logcosh(kb*db/2)-log(2))
  ll <- sum(ys*eta-pmax(eta,0)-log1p(exp(-abs(eta)))+within)
  gradbeta <- function(dm,dsx,dbx)ys*ks/2*(ss-ts)*dsx+(1-ys)*kb/2*(sb-tb)*dbx+
    res*(dm+ts*dsx/2-tb*dbx/2)
  score <- cbind(gradbeta(dmp,dps,dpb)/100,gradbeta(dmq,dqs,dqb),res,res*y,
    ys*ds/2*(ss-ts)+res*entropic_derivative(ks,ds),
    ((1-ys)*db/2*(sb-tb)-res*entropic_derivative(kb,db))*.05)
  ps <- plogis(ks*ds);pb <- plogis(kb*db)
  list(ll=ll,g=colSums(score),prob=cbind(S*ps,S*(1-ps),(1-S)*pb,(1-S)*(1-pb)))
}
# L-BFGS status is retained, never accepted as a convergence certificate. Polish
# free directions with the observed derivative matrix, check positive curvature,
# score/step and KKT signs of active reciprocal-scale bounds independently.
fit <- function(start,shore=NULL) {
  free <- if(is.null(shore))1:6 else c(1:4,6)
  unpack <- function(par){theta <- start;theta[free] <- par;if(!is.null(shore))theta[5] <- shore;theta}
  lower <- rep(-Inf,length(free));lower[free==6] <- 0
  if(is.null(shore))lower[free==5] <- 0
  opt <- optim(start[free],function(v)-evaluate(unpack(v))$ll/N,
    function(v)-evaluate(unpack(v))$g[free]/N,method='L-BFGS-B',lower=lower,
    control=list(maxit=2000,factr=1,pgtol=1e-11,lmm=20))
  theta <- unpack(opt$par)
  active <- c(if(is.null(shore)&&theta[5]==0)5 else integer(),if(theta[6]==0)6 else integer())
  polish_free <- setdiff(free,active)
  for(iteration in 1:15) {
    at <- function(v){candidate <- theta;candidate[polish_free] <- v;candidate}
    H <- optimHess(theta[polish_free],function(v)-evaluate(at(v))$ll/N,
      function(v)-evaluate(at(v))$g[polish_free]/N,control=list(ndeps=rep(1e-4,length(polish_free))))
    H <- (H+t(H))/2
    stopifnot(min(eigen(H,symmetric=TRUE,only.values=TRUE)$values)>0)
    e <- evaluate(theta);step <- drop(solve(H,e$g[polish_free]/N))
    if(max(abs(e$g[polish_free]))/N<1e-11 && max(abs(step))<1e-10)break
    candidate <- theta;candidate[polish_free] <- theta[polish_free]+step
    stopifnot(candidate[5]>=0,candidate[6]>=0,evaluate(candidate)$ll>=e$ll-1e-9)
    theta <- candidate
  }
  e <- evaluate(theta)
  stopifnot(max(abs(e$g[polish_free]))/N<1e-11,max(abs(step))<1e-10,all(e$g[active]<=1e-8))
  list(theta=theta,ll=e$ll,g=e$g,active=active,code=opt$convergence,
    free_score=max(abs(e$g[polish_free]))/N,free_step=max(abs(step)))
}
emit_fit <- function(section,fit,i=0,j=0) {
  emit(section,'log_likelihood',fit$ll,i,j)
  emit(section,'ks',fit$theta[5],i,j);emit(section,'kb',.05*fit$theta[6],i,j)
  emit(section,'active_shore',as.numeric(5%in%fit$active),i,j)
  emit(section,'active_boat',as.numeric(6%in%fit$active),i,j)
  emit(section,'free_score',fit$free_score,i,j);emit(section,'free_step',fit$free_step,i,j)
  emit(section,'raw_lbfgs_code',fit$code,i,j)
}
initial <- c(100*beta[1],beta[2:4],1/beta[5],1/beta[5]/.05)
conditional <- c(100*restricted[1],restricted[2:4],1,20)
d <- do.call(rbind,lapply(seq_len(N),function(i)data.frame(id=i,alternative=1:4,
  chosen=w$mode[i]==1:4,p=p[i,],q=q[i,],dshore=c(1,1,0,0),dshorey=c(1,1,0,0)*y[i])))
X <- lapply(1:4,function(j)as.matrix(d[d$alternative==j,c('p','q','dshore','dshorey')]))
Y <- lapply(1:4,function(j)d$chosen[d$alternative==j]);names(X) <- names(Y) <- as.character(1:4)
for(i in 1:4) {
  theta <- initial;theta[5] <- c(.005,.01,.04,1)[i];ks <- theta[5];kb <- .05*theta[6]
  par <- c(theta[1]/100,theta[2],theta[3]-log(2)*(1/ks-1/kb),theta[4],1/ks,1/kb)
  pkg <- getFromNamespace('lnl.nlogit','mlogit')(par,X,Y,weights=rep(1,N),gradient=TRUE,
    opposite=1,nests=list(shore=c('1','2'),boat=c('3','4')),unscaled=FALSE,un.nest.el=FALSE)
  ours <- evaluate(theta);g <- attr(pkg,'gradient')
  mapped <- c(g[1]/100,g[2:4],(log(2)*g[3]-g[5])/ks^2,(-log(2)*g[3]-g[6])*.05/kb^2)
  delta <- c(ll=abs(ours$ll-as.numeric(pkg)),score=max(abs(ours$g-mapped)),
    probability=max(abs(ours$prob-attr(pkg,'probabilities'))))
  stopifnot(delta[1]<1e-8,delta[2]<1e-6,delta[3]<1e-12)
  for(name in names(delta))emit('package_control',name,delta[name],i)
}
starts <- list(initial,conditional)
for(bq in c(-1,1,3))for(ks in c(.01,1,5))starts[[length(starts)+1L]] <- c(-2,bq,-.5,-.1,ks,2)
fits <- lapply(starts,fit);boundary <- fits[[which.max(vapply(fits,function(f)f$ll,0.))]]
stopifnot(boundary$theta[5]==0,boundary$theta[6]>0)
for(i in seq_along(fits)) {
  stopifnot(fits[[i]]$theta[5]==0,abs(fits[[i]]$ll-boundary$ll)<1e-9)
  emit_fit('multistart',fits[[i]],i)
}
previous <- initial;grid <- c(0,.0001,.001,.005,.01,.02,.04,.1,.25,.5,1,2)
for(i in seq_along(grid)) {
  candidates <- lapply(list(previous,initial,conditional),function(start)fit(start,shore=grid[i]))
  for(j in seq_along(candidates))emit_fit('profile_candidate',candidates[[j]],i,j)
  best <- candidates[[which.max(vapply(candidates,function(f)f$ll,0.))]]
  emit_fit('profile',best,i);emit('profile','shore_score',best$g[5],i)
  stopifnot(best$ll<=boundary$ll+1e-9,best$g[5]<0)
  previous <- best$theta
}
# At ks=0, entropic(ks,ds)=ks*ds^2/8+O(ks^3). The following independent
# expression therefore has the exact value and full first/second derivatives
# at this boundary. No finite-scale truncated approximation is used above.
theta <- boundary$theta;ds <- quote(pc/100*dps+bq*dqs);db <- quote(pc/100*dpb+bq*dqb)
eta <- substitute(a+b*y+pc/100*dmp+bq*dmq+ks*ds^2/8-
  log((exp(.05*bk*db/2)+exp(-.05*bk*db/2))/2)/(.05*bk),list(ds=ds,db=db))
expr <- substitute(ys*eta-log(1+exp(eta))+
  ys*(ss*ks*ds/2-log((exp(ks*ds/2)+exp(-ks*ds/2))/2)-log(2))+
  (1-ys)*(sb*.05*bk*db/2-log((exp(.05*bk*db/2)+exp(-.05*bk*db/2))/2)-log(2)),list(eta=eta,ds=ds,db=db))
fn <- deriv3(as.expression(expr),c('pc','bq','a','b','ks','bk'),function.arg=TRUE)
v <- do.call(fn,as.list(theta));sg <- colSums(attr(v,'gradient'));H <- apply(attr(v,'hessian'),c(2,3),sum)
e <- evaluate(theta);free <- c(1:4,6)
minimum <- min(eigen(-H[free,free],symmetric=TRUE,only.values=TRUE)$values)
step <- solve(-H[free,free],sg[free])
stopifnot(abs(sum(v)-e$ll)<1e-9,max(abs(sg-e$g))<1e-9,minimum>0,
  max(abs(sg[free]))/N<1e-11,max(abs(step))<1e-10,sg[5] < -2)
for(i in 1:6)emit('boundary','theta',theta[i],i)
for(name in c('ll_difference','score_difference','minimum_free_information','free_step','shore_score','boat_lambda'))
  emit('boundary',name,switch(name,ll_difference=abs(sum(v)-e$ll),score_difference=max(abs(sg-e$g)),
    minimum_free_information=minimum,free_step=max(abs(step)),shore_score=sg[5],boat_lambda=1/(.05*theta[6])))
for(i in 1:3) {
  epsilon <- c(1e-3,1e-4,1e-5)[i]
  numeric <- optimHess(theta,function(t)-evaluate(t)$ll,function(t)-evaluate(t)$g,control=list(ndeps=rep(epsilon,6)))
  delta <- max(abs(numeric+H)/(1+abs(H)));stopifnot(delta<2e-5)
  emit('boundary_hessian','step',epsilon,i);emit('boundary_hessian','relative_error',delta,i)
}
# Historical source coordinates and precision controls. A vertex sweep is a
# sensitivity check, NOT an interval certificate over every interior box point.
f32 <- function(x)readBin(writeBin(as.double(x),raw(),size=4),double(),n=length(x),size=4)
set_data <- function(single=FALSE) {
  z <- w;if(single)z[] <- lapply(z,f32)
  assign('y',if(single)f32(z$income/1000)else z$income/1000,envir=.GlobalEnv)
  assign('shore',as.numeric(z$mode<=2),envir=.GlobalEnv)
  assign('px',as.matrix(z[pnames])[cbind(seq_len(N),z$mode)],envir=.GlobalEnv)
  assign('qx',as.matrix(z[qnames])[cbind(seq_len(N),z$mode)],envir=.GlobalEnv)
  for(j in 1:4){assign(paste0('p',j),z[[pnames[j]]],envir=.GlobalEnv);assign(paste0('q',j),z[[qnames[j]]],envir=.GlobalEnv)}
}
expr <- expression(bp*px+bq*qx+shore*(a+b*y)+
  shore*(rs-1)*log(exp(bp*p1+bq*q1)+exp(bp*p2+bq*q2))+
  (1-shore)*(rb-1)*log(exp(bp*p3+bq*q3)+exp(bp*p4+bq*q4))-
  log(exp(a+b*y)*(exp(bp*p1+bq*q1)+exp(bp*p2+bq*q2))^rs+
    (exp(bp*p3+bq*q3)+exp(bp*p4+bq*q4))^rb))
fn <- deriv3(expr,c('bp','bq','a','b','rs','rb'),function.arg=TRUE)
D <- diag(c(.01,1,1,1,30,15))
audit <- function(par) {
  value <- do.call(fn,as.list(unname(par)));g <- colSums(attr(value,'gradient'))
  H <- apply(attr(value,'hessian'),c(2,3),sum);information <- -D%*%H%*%D
  c(log_likelihood=sum(value),scaled_score=max(abs(D%*%g))/N,
    scaled_step=max(abs(solve(information,drop(D%*%g)))),
    minimum_information=min(eigen(information,symmetric=TRUE,only.values=TRUE)$values))
}
stata <- c(-.0013303,.1284825,-11.40196,.1108341,29.98591,14.06438)
l <- c(-.001240447647,.1200298028,-.3836841537,.007377036421,32.17258445,15.07475969)
# Original LIMDEP dshorey=1-(1-shore)*y is a RAW lower nest-constant feature.
# Its case-common component cannot be dropped with unequal nonnormalized scales.
limdep <- c(l[1:2],l[5]*l[3]+(l[5]-l[6])*l[4],l[6]*l[4],l[5:6])
for(i in 1:6){emit('historical_stata','parameter',stata[i],i);emit('historical_limdep','parameter',limdep[i],i)}
half_units <- c(5e-8,5e-8,5e-6,5e-8,5e-6,5e-6)
signs <- as.matrix(expand.grid(rep(list(c(-1,1)),6)))
for(single in c(FALSE,TRUE)) {
  set_data(single);j <- as.integer(single)
  for(i in 1:3) {
    result <- audit(list(stationary,stata,limdep)[[i]])
    stopifnot(result['minimum_information']>0)
    if(!single && i==1) {
      stopifnot(result['scaled_score']<1e-10,result['scaled_step']<1e-10)
      control_ll <- result['log_likelihood']
    }
    if(i==2)stopifnot(round(result['log_likelihood'],4)==-1180.0156,
      result['scaled_score']>1e-8,result['scaled_step']>1e-3)
    if(!single && i==3)stopifnot(abs(result['log_likelihood']-control_ll)<1e-8)
    for(name in names(result))emit('historical_point',name,result[name],i,j)
  }
  for(i in 1:64) {
    result <- audit(stata+signs[i,]*half_units)
    stopifnot(result['scaled_score']>1e-8,result['scaled_step']>1e-3,result['minimum_information']>0)
    for(name in names(result))emit('historical_rounding_vertex',name,result[name],i,j)
  }
}
# Emit only after all checks complete: validation failure emits no diagnostics.
# The checker also requires the complete table. No Wald covariance/LR is emitted.
write.csv(do.call(rbind,records),stdout(),row.names=FALSE,quote=FALSE)
