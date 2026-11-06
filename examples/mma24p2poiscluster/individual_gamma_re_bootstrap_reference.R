# Development-only independent integrated-likelihood refits. No native optimizer is used.
options(digits=17)
a <- commandArgs(trailingOnly=TRUE)
if(length(a)<3 || length(a)>4 || (length(a)==4 && !a[4]%in%c('--csv','--dta')))
  stop('usage: individual_gamma_re_bootstrap_reference.R source oracle-draws.csv output-directory [--csv | --dta]')
if(file.exists(a[3])) stop('reference directory already exists')
mode <- if(length(a)==4) a[4] else 'ascii'
if(mode=='--csv') {
  source <- read.csv(a[1])
} else if(mode=='--dta') {
  d <- foreign::read.dta(a[1],convert.factors=FALSE)
  stopifnot(nrow(d)==27766,identical(which(is.na(d$PHARVIS)),27734L),all(is.na(d[27734,])))
  source <- data.frame(source_row=seq_len(nrow(d)),pharvis=d$PHARVIS,lnhhexp=d$lnhhinc,
    age=d$AGE,sex=d$SEX,married=d$MARRIED,educ=d$COMPED98,illness=d$ILLDUM,
    injury=d$INJDUM,illdays=d$ILLDAYS,actdays=d$ACTDAYS,insurance=d$HLTHINS,commune=d$commune)
} else {
  tokens <- scan(a[1],what=character(),quiet=TRUE);stopifnot(length(tokens)==27766*12)
  raw <- matrix(tokens,ncol=12,byrow=TRUE);values <- matrix(as.numeric(replace(raw,raw=='.',NA)),ncol=12)
  stopifnot(identical(which(rowSums(is.na(values))>0),27734L),all(is.na(values[27734,])))
  source <- data.frame(source_row=seq_len(nrow(values)),values)
  names(source) <- c('source_row','pharvis','lnhhexp','age','sex','married','educ','illness','injury','illdays','actdays','insurance','commune')
}
predictors <- c('lnhhexp','insurance','sex','age','married','illdays','actdays','injury','illness','educ')
required <- c('source_row','pharvis',predictors,'commune')
stopifnot(all(required%in%names(source)),all(vapply(source[required],is.numeric,logical(1))),
  nrow(source)>0,nrow(source)<=27766,all(is.finite(source$source_row)),
  all(source$source_row>=1 & source$source_row<=27766),all(source$source_row==floor(source$source_row)),!anyDuplicated(source$source_row))
missing <- is.na(source$pharvis)
stopifnot(all(vapply(source[missing,setdiff(required,'source_row'),drop=FALSE],function(v)all(is.na(v)),logical(1))))
d <- source[!missing,];d <- d[order(d$source_row),]
stopifnot(all(vapply(d[required],function(v)all(is.finite(v)),logical(1))),
  all(d$pharvis>=0 & d$pharvis<=100000 & d$pharvis==floor(d$pharvis)),
  all(d$commune>=1 & d$commune<=194 & d$commune==floor(d$commune)))
ids <- unique(d$commune);grp <- match(d$commune,ids);G <- length(ids);sizes <- tabulate(grp,nbins=G)
X <- cbind('_cons'=1,as.matrix(d[predictors]));y <- d$pharvis;K <- ncol(X)
S <- as.numeric(rowsum(y,grp,reorder=FALSE));stopifnot(G>K+1,G<=194,max(sizes)<=206,all(S<=100000),K==11)
scales <- sqrt(colMeans(X^2));Z <- sweep(X,2,scales,'/')
draws <- read.csv(a[2]);B <- max(draws$replicate)
stopifnot(identical(names(draws),c('replicate','copy_id','cluster_index','commune')),
  B>=2,nrow(draws)==B*G,identical(draws$replicate,rep(seq_len(B),each=G)),
  identical(draws$copy_id,rep(seq_len(G),times=B)),all(draws$cluster_index>=0 & draws$cluster_index<G),
  all(draws$commune==ids[draws$cluster_index+1L]))
# Independent direct special functions, vectorized by original group. Multiplicity
# weights each complete group likelihood, equivalent to distinct shared effects.
evaluate <- function(p,multiplicity,information=FALSE) {
  r <- exp(-p[K+1]);eta <- drop(Z%*%p[1:K]);mu <- exp(eta)
  if(!is.finite(r) || abs(p[K+1])>=12 || any(!is.finite(mu)) || any(mu<=0))
    return(list(ll=-1e100,score=rep(0,K+1)))
  M <- as.numeric(rowsum(mu,grp,reorder=FALSE));q <- (r+S)/(r+M)
  dg <- digamma(r+S)-digamma(r)+log(r)+1-log(r+M)-(r+S)/(r+M)
  group_ll <- lgamma(r+S)-lgamma(r)+r*log(r)-(r+S)*log(r+M)
  ll <- sum(multiplicity*group_ll)+sum(multiplicity[grp]*(y*eta-lgamma(y+1)))
  score <- c(drop(crossprod(Z,multiplicity[grp]*(y-q[grp]*mu))),sum(multiplicity*(-r*dg)))
  if(!information) return(list(ll=ll,score=score,mu=mu,eta=eta))
  h <- rowsum(Z*mu,grp,reorder=FALSE)
  info <- matrix(0,K+1,K+1)
  info[1:K,1:K] <- crossprod(Z*sqrt(multiplicity[grp]*q[grp]*mu))-crossprod(h*sqrt(multiplicity*q/(r+M)))
  cross <- colSums(h*(-multiplicity*r*(M-S)/(r+M)^2));info[1:K,K+1] <- cross;info[K+1,1:K] <- cross
  dgp <- trigamma(r+S)-trigamma(r)+1/r-1/(r+M)+(S-M)/(r+M)^2
  info[K+1,K+1] <- sum(multiplicity*(-r*dg-r*r*dgp))
  list(ll=ll,score=score,info=info,mu=mu,eta=eta)
}
refit <- function(multiplicity,start) {
  opt <- optim(start,function(p)-evaluate(p,multiplicity)$ll/G,
    function(p)-evaluate(p,multiplicity)$score/G,method='BFGS',control=list(reltol=1e-13,maxit=2000))
  stopifnot(opt$convergence==0);p <- opt$par
  for(i in seq_len(15)) {
    value <- evaluate(p,multiplicity,TRUE);step <- solve(value$info,value$score)
    stopifnot(all(is.finite(step)))
    if(max(abs(step))<1e-12 && max(abs(value$score))<1e-7) break
    fraction <- 1
    for(attempt in seq_len(40)) {
      trial <- p+fraction*step
      if(evaluate(trial,multiplicity)$ll >= value$ll-1e-12*(1+abs(value$ll))) break
      fraction <- fraction/2
    }
    stopifnot(attempt<40);p <- trial
  }
  value <- evaluate(p,multiplicity,TRUE)
  stopifnot(i<15,max(abs(value$score))<1e-7,min(eigen(value$info,symmetric=TRUE)$values)>0,abs(p[K+1])<12)
  p
}
pooled <- glm.fit(Z,y,family=poisson(),control=glm.control(epsilon=1e-13,maxit=100))
stopifnot(pooled$converged,pooled$rank==K);base <- refit(rep(1,G),c(coef(pooled),0))
bs <- matrix(NA_real_,nrow=B,ncol=K+1);counts <- integer(B)
for(i in seq_len(B)) {
  chosen <- draws$cluster_index[draws$replicate==i]+1L
  multiplicity <- tabulate(chosen,nbins=G);counts[i] <- sum(sizes[chosen])
  bs[i,] <- refit(multiplicity,base)/c(scales,1)
}
point <- unname(base/c(scales,1));names <- c('_cons',predictors,'lnalpha')
V <- cov(bs);means <- colMeans(bs);ci <- t(apply(bs,2,quantile,probs=c(.025,.975),type=7,names=FALSE))
fits <- data.frame(replicate=seq_len(B),sample_n=counts,bs,check.names=FALSE);colnames(fits) <- c('replicate','sample_n',names)
summary <- do.call(rbind,lapply(seq_along(names),function(i)do.call(rbind,lapply(seq_along(names),function(j)
  data.frame(parameter_index=i,covariance_index=j,parameter=names[i],covariance_parameter=names[j],estimate=point[i],
    bootstrap_mean=means[i],bias=means[i]-point[i],standard_error=sqrt(V[i,i]),ci_lower=ci[i,1],ci_upper=ci[i,2],covariance=V[i,j])))))
d$selected_commune_total <- S[grp];d$included_in_re <- TRUE
value <- evaluate(base,rep(1,G));d$re_mean <- value$mu;d$re_xb <- value$eta;d$re_residual <- y-value$mu
if(!dir.create(a[3],recursive=TRUE)) stop('cannot create reference directory')
write_precise <- function(data,path) {
  numeric_columns <- vapply(data,is.numeric,logical(1))
  data[numeric_columns] <- lapply(data[numeric_columns],function(x)ifelse(is.na(x),NA_character_,sprintf('%.17g',x)))
  write.csv(data,path,row.names=FALSE,na='')
}
write_precise(source,file.path(a[3],'source.csv'));write_precise(d,file.path(a[3],'predictions.csv'))
write_precise(fits,file.path(a[3],'fits.csv'));write_precise(summary,file.path(a[3],'summary.csv'))
write_precise(data.frame(commune=ids,observations=sizes),file.path(a[3],'groups.csv'))
cat(R.version.string,'N',nrow(d),'G',G,'B',B,'replicate N range',range(counts),'\n')
