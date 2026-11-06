# Development-only independent conditional-score refits; no Rust optimizer or production R dependency.
options(digits=17)
a <- commandArgs(trailingOnly=TRUE)
if (length(a)<3 || length(a)>4 || (length(a)==4 && !a[4]%in%c('--csv','--dta')))
  stop('usage: individual_conditional_fe_bootstrap_reference.R source oracle-draws.csv output-directory [--csv | --dta]')
if (file.exists(a[3])) stop('reference directory already exists')
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
  raw <- matrix(tokens,ncol=12,byrow=TRUE)
  values <- matrix(as.numeric(replace(raw,raw=='.',NA)),ncol=12)
  stopifnot(identical(which(rowSums(is.na(values))>0),27734L),all(is.na(values[27734,])))
  source <- data.frame(source_row=seq_len(nrow(values)),values)
  names(source) <- c('source_row','pharvis','lnhhexp','age','sex','married','educ',
    'illness','injury','illdays','actdays','insurance','commune')
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
totals <- tapply(d$pharvis,d$commune,sum)
d$selected_commune_total <- unname(totals[as.character(d$commune)])
d$included_in_fe <- d$selected_commune_total>0
selected <- d[d$included_in_fe,];ids <- unique(selected$commune)
grp <- match(selected$commune,ids);G <- length(ids);X <- as.matrix(selected[predictors]);y <- selected$pharvis
sizes <- tabulate(grp,nbins=G);K <- ncol(X);total <- as.numeric(rowsum(y,grp,reorder=FALSE))
stopifnot(G>K,G<=194,max(sizes)<=206,K==10,all(total>0))
draws <- read.csv(a[2]);B <- max(draws$replicate)
stopifnot(identical(names(draws),c('replicate','copy_id','cluster_index','commune')),
  B>=2,nrow(draws)==B*G,identical(draws$replicate,rep(seq_len(B),each=G)),
  identical(draws$copy_id,rep(seq_len(G),times=B)),all(draws$cluster_index>=0 & draws$cluster_index<G),
  all(draws$commune==ids[draws$cluster_index+1L]))
# Repeated identical group copies contribute multiplicity times the same conditional
# score/information. This is algebraically equivalent to distinct raw-copy IDs.
# The separate synthetic GLM check independently tests that equivalence.
calculate <- function(beta,multiplicity) {
  eta <- drop(X%*%beta);anchor <- as.numeric(tapply(eta,grp,max))
  w <- exp(eta-anchor[grp]);den <- as.numeric(rowsum(w,grp,reorder=FALSE))
  mu <- total[grp]*w/den[grp];means <- rowsum(X*w,grp,reorder=FALSE)/den
  score <- drop(crossprod(X,(y-mu)*multiplicity[grp]))
  info <- crossprod(X*sqrt(mu*multiplicity[grp]))-crossprod(means*sqrt(total*multiplicity))
  objective <- sum(multiplicity[grp]*y*((eta-anchor[grp])-log(den[grp])))
  stopifnot(all(is.finite(score)),all(is.finite(info)),all(is.finite(mu)),is.finite(objective))
  list(score=score,info=info,mu=mu,objective=objective)
}
refit <- function(multiplicity) {
  beta <- rep(0,K)
  for(iteration in seq_len(100)) {
    value <- calculate(beta,multiplicity);step <- solve(value$info,value$score)
    stopifnot(all(is.finite(step)))
    if(max(abs(step))<1e-12 && max(abs(value$score))<1e-7) break
    damping <- 1
    for(attempt in seq_len(40)) {
      trial <- beta+damping*step
      objective <- calculate(trial,multiplicity)$objective
      if(objective >= value$objective-1e-12*(1+abs(value$objective))) break
      damping <- damping/2
    }
    stopifnot(attempt<40);beta <- trial
  }
  stopifnot(iteration<100,max(abs(calculate(beta,multiplicity)$score))<1e-7)
  beta
}
base <- refit(rep(1,G));bs <- matrix(NA_real_,nrow=B,ncol=K);counts <- integer(B)
for(r in seq_len(B)) {
  chosen <- draws$cluster_index[draws$replicate==r]+1L
  multiplicity <- tabulate(chosen,nbins=G);counts[r] <- sum(sizes[chosen]);bs[r,] <- refit(multiplicity)
}
V <- cov(bs);means <- colMeans(bs);ci <- t(apply(bs,2,quantile,probs=c(.025,.975),type=7,names=FALSE))
fits <- data.frame(replicate=seq_len(B),sample_n=counts,bs,check.names=FALSE)
colnames(fits) <- c('replicate','sample_n',predictors)
summary <- do.call(rbind,lapply(seq_len(K),function(i)do.call(rbind,lapply(seq_len(K),function(j)
  data.frame(parameter_index=i,covariance_index=j,parameter=predictors[i],covariance_parameter=predictors[j],
    estimate=base[i],bootstrap_mean=means[i],bias=means[i]-base[i],standard_error=sqrt(V[i,i]),
    ci_lower=ci[i,1],ci_upper=ci[i,2],covariance=V[i,j])))))
d$fe_xb <- drop(as.matrix(d[predictors])%*%base)
d$fe_mean <- NA_real_;d$fe_mean[d$included_in_fe] <- calculate(base,rep(1,G))$mu
d$fe_residual <- d$pharvis-d$fe_mean
if(!dir.create(a[3],recursive=TRUE)) stop('cannot create reference directory')
write_precise <- function(data,path) {
  numeric_columns <- vapply(data,is.numeric,logical(1))
  data[numeric_columns] <- lapply(data[numeric_columns],function(x)ifelse(is.na(x),NA_character_,sprintf('%.17g',x)))
  write.csv(data,path,row.names=FALSE,na='')
}
write_precise(source,file.path(a[3],'source.csv'));write_precise(d,file.path(a[3],'predictions.csv'))
write_precise(fits,file.path(a[3],'fits.csv'));write_precise(summary,file.path(a[3],'summary.csv'))
write_precise(data.frame(commune=ids,observations=sizes),file.path(a[3],'groups.csv'))
cat(R.version.string,'N',nrow(selected),'G',G,'B',B,'replicate N range',range(counts),'\n')
