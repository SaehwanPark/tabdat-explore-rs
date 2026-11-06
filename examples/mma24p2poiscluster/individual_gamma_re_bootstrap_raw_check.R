# Independent raw-copy group likelihood versus multiplicity R reference.
# A merged-source-ID sensitivity fit is a different gamma likelihood.
a <- commandArgs(trailingOnly=TRUE)
if(length(a)!=3) stop('usage: individual_gamma_re_bootstrap_raw_check.R synthetic.csv oracle-draws.csv reference-dir')
d <- read.csv(a[1]);d <- d[!is.na(d$pharvis),];d <- d[order(d$source_row),]
vars <- c('lnhhexp','insurance','sex','age','married','illdays','actdays','injury','illness','educ')
ids <- unique(d$commune);G <- length(ids);stopifnot(G==48,nrow(d)==278)
draws <- read.csv(a[2]);ref <- read.csv(file.path(a[3],'fits.csv'),check.names=FALSE)
stopifnot(nrow(draws)==500*G,nrow(ref)==500)
fit_raw <- function(z,key) {
  X <- cbind('_cons'=1,as.matrix(z[vars]));y <- z$pharvis;K <- ncol(X)
  groups <- split(seq_len(nrow(z)),key);S <- vapply(groups,function(i)sum(y[i]),0.0)
  scales <- sqrt(colMeans(X^2));Z <- sweep(X,2,scales,'/');pooled <- glm.fit(Z,y,family=poisson(),control=glm.control(epsilon=1e-13,maxit=100))
  stopifnot(pooled$converged,pooled$rank==K)
  evaluate <- function(p) {
    r <- exp(-p[K+1]);eta <- drop(Z%*%p[1:K]);mu <- exp(eta)
    if(!is.finite(r) || abs(p[K+1])>=12 || any(!is.finite(mu)) || any(mu<=0)) return(list(ll=-1e100,score=rep(0,K+1)))
    score <- rep(0,K+1);info <- matrix(0,K+1,K+1);ll <- 0
    for(j in seq_along(groups)) {
      i <- groups[[j]];x <- Z[i,,drop=FALSE];m <- mu[i];M <- sum(m);s <- S[j];q <- (r+s)/(r+M);h <- drop(crossprod(x,m))
      ll <- ll+lgamma(r+s)-lgamma(r)+r*log(r)-(r+s)*log(r+M)+sum(y[i]*eta[i]-lgamma(y[i]+1))
      dg <- digamma(r+s)-digamma(r)+log(r)+1-log(r+M)-(r+s)/(r+M)
      dgp <- trigamma(r+s)-trigamma(r)+1/r-1/(r+M)+(s-M)/(r+M)^2
      score[1:K] <- score[1:K]+drop(crossprod(x,y[i]-q*m));score[K+1] <- score[K+1]-r*dg
      info[1:K,1:K] <- info[1:K,1:K]+q*crossprod(x,m*x)-q/(r+M)*tcrossprod(h)
      info[1:K,K+1] <- info[1:K,K+1]-r*(M-s)/(r+M)^2*h;info[K+1,1:K] <- info[1:K,K+1]
      info[K+1,K+1] <- info[K+1,K+1]-r*dg-r*r*dgp
    }
    list(ll=ll,score=score,info=info)
  }
  opt <- optim(c(coef(pooled),0),function(p)-evaluate(p)$ll/G,function(p)-evaluate(p)$score/G,method='BFGS',control=list(reltol=1e-13,maxit=2000))
  stopifnot(opt$convergence==0);p <- opt$par
  for(it in 1:12) {e <- evaluate(p);step <- solve(e$info,e$score);p <- p+step;if(max(abs(step))<1e-12)break}
  e <- evaluate(p);stopifnot(max(abs(e$score))<1e-7,min(eigen(e$info,symmetric=TRUE)$values)>0)
  unname(p/c(scales,1))
}
for(r in c(1,250,500)) {
  chosen <- draws$cluster_index[draws$replicate==r]+1L
  stopifnot(length(chosen)==G,length(unique(chosen))<G)
  pieces <- lapply(seq_along(chosen),function(i){z<-d[d$commune==ids[chosen[i]],];z$copy_id<-i;z})
  copied <- do.call(rbind,pieces);stopifnot(any(table(copied$copy_id)==1),any(tapply(copied$pharvis,copied$copy_id,sum)==0))
  actual <- fit_raw(copied,copied$copy_id);expected <- as.numeric(ref[r,c('_cons',vars,'lnalpha')])
  diff <- max(abs(actual-expected));stopifnot(diff<=1e-9)
  cat('replicate',r,'expanded distinct-copy gamma likelihood versus multiplicity reference max difference',format(diff,digits=6),'\n')
  if(r==1) {
    merged <- fit_raw(copied,copied$commune);delta <- max(abs(merged-actual));stopifnot(delta>1e-3)
    cat('replicate 1 merged-source-ID versus distinct-copy max parameter difference',format(delta,digits=6),'(not historical replay)\n')
  }
}
