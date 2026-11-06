# R QR refits independently expanded raw commune copies and recomputes RE components.
options(digits=17)
a <- commandArgs(trailingOnly=TRUE)
if(length(a)<3 || length(a)>4 || (length(a)==4 && !a[4]%in%c('--csv','--dta')))
  stop('usage: household_re_bootstrap_reference.R source native-draws.csv output-directory [--csv | --dta]')
if(file.exists(a[3])) stop('reference directory already exists')
mode <- if(length(a)==4) a[4] else 'ascii'
if(mode=='--csv') {
  source <- read.csv(a[1])
} else if(mode=='--dta') {
  raw <- foreign::read.dta(a[1],convert.factors=FALSE)
  stopifnot(nrow(raw)==5999,sum(is.na(raw$lhhex12m))==993)
  source <- data.frame(source_row=seq_len(nrow(raw)),lnexp12m=raw$lhhex12m,
      age=raw$age,sex=raw$sex,hhsize=raw$hhsize,farm=raw$farm,
      educ=raw$comped98,lnhhexp=raw$lhhexp1,commune=raw$commune)
} else {
  tokens <- scan(a[1],what=character(),quote='"',quiet=TRUE)
  stopifnot(length(tokens)==5999*8)
  raw <- matrix(tokens,ncol=8,byrow=TRUE)
  farms <- c('non farm'=0,'farm'=1)
  education <- c('Never'=0,'Nhatre'=1,'<cap I'=2,'Cap I'=3,'Cap II'=4,
      'Cap III'=5,'Nghe SC'=6,'THCN'=7,'DHDC'=8,'DHCD'=9,
      'Thac sy'=10,'PTS'=11,'TS'=12)
  num <- function(x) as.numeric(replace(x,x=='.',NA))
  source <- data.frame(source_row=seq_len(nrow(raw)),lnexp12m=num(raw[,1]),
      age=num(raw[,2]),sex=num(raw[,3]),hhsize=num(raw[,4]),
      farm=unname(farms[raw[,5]]),educ=unname(education[raw[,6]]),
      lnhhexp=num(raw[,7]),commune=num(raw[,8]))
  stopifnot(!anyNA(source[,-2]),sum(is.na(source$lnexp12m))==993)
}
predictors <- c('lnhhexp','age','sex','hhsize','farm','educ')
required <- c('source_row','lnexp12m',predictors,'commune')
stopifnot(all(required%in%names(source)),all(vapply(source[required],is.numeric,logical(1))),
    nrow(source)>0,nrow(source)<=5999,all(is.finite(source$source_row)),
    all(source$source_row>=1 & source$source_row<=5999),all(source$source_row==floor(source$source_row)),!anyDuplicated(source$source_row),
    all(is.finite(source$commune)),all(source$commune>=1 & source$commune<=194),all(source$commune==floor(source$commune)),
    all(vapply(source[predictors],function(v)all(is.finite(v)&abs(v)<=1e6),logical(1))),
    !any(is.nan(source$lnexp12m)),all(is.na(source$lnexp12m)|(is.finite(source$lnexp12m)&abs(source$lnexp12m)<=1e6)))
d <- source[!is.na(source$lnexp12m),];d <- d[order(d$source_row),]
ids <- sort(unique(d$commune));G <- length(ids);B <- 500L
members <- lapply(ids,function(id)which(d$commune==id));sizes <- lengths(members)
stopifnot(G>=8,G<=194,max(sizes)<=39,nrow(d)>G+6)
X <- as.matrix(d[predictors]);y <- d$lnexp12m
raw_draws <- read.csv(a[2]);stopifnot(identical(names(raw_draws),c('replicate','copy_id','step','commune')),
    nrow(raw_draws)==B*G,identical(raw_draws$replicate,rep(seq_len(B),each=G)),
    identical(raw_draws$copy_id,rep(seq_len(G),times=B)),all(raw_draws$step>=raw_draws$copy_id),
    all(raw_draws$commune%in%ids))
# Distinct copied IDs are essential even when a source commune is selected repeatedly.
refit <- function(chosen) {
  index <- unlist(members[chosen],use.names=FALSE)
  group <- rep(seq_along(chosen),lengths(members[chosen]))
  n <- length(index);K <- length(chosen);stopifnot(K>7,n>K+6)
  x <- X[index,,drop=FALSE];ycopy <- y[index]
  sizes <- lengths(members[chosen]);centres <- rowsum(x,group,reorder=FALSE)/sizes
  ycentres <- drop(rowsum(ycopy,group,reorder=FALSE))/sizes
  within <- lm.fit(x-centres[group,,drop=FALSE],ycopy-ycentres[group]);stopifnot(within$rank==6,all(is.finite(within$coefficients)))
  between <- lm.fit(cbind(1,centres),ycentres);stopifnot(between$rank==7,all(is.finite(between$coefficients)))
  within_rss <- sum(within$residuals^2);between_rss <- sum(between$residuals^2)
  e <- within_rss/(n-K-6);raw_u <- between_rss/(K-7)-e*mean(1/sizes);u <- max(0,raw_u)
  stopifnot(is.finite(e),e>0,is.finite(raw_u),is.finite(u))
  theta <- 1-sqrt(e/(e+sizes*u));stopifnot(all(is.finite(theta)),all(theta>=0),all(theta<1))
  xq <- cbind(1-theta[group],x-theta[group]*centres[group,,drop=FALSE])
  yq <- ycopy-theta[group]*ycentres[group]
  gls <- lm.fit(xq,yq);stopifnot(gls$rank==7,all(is.finite(gls$coefficients)))
  diagnostics <- c(e=e,raw_u=raw_u,u=u,within_rss=within_rss,between_rss=between_rss,
      mean_inverse_n=mean(1/sizes),theta_min=min(theta),theta_max=max(theta),
      setNames(unname(within$coefficients),paste0('within_',predictors)),
      setNames(unname(between$coefficients[-1]),paste0('between_',predictors)))
  list(coefficients=unname(gls$coefficients),n=n,diagnostics=diagnostics)
}
base <- refit(seq_len(G));bs <- matrix(NA_real_,nrow=B,ncol=7);counts <- integer(B)
diagnostics <- matrix(NA_real_,nrow=B,ncol=20)
for(rep in seq_len(B)) {
  chosen <- match(raw_draws$commune[raw_draws$replicate==rep],ids)
  fit <- refit(chosen);bs[rep,] <- fit$coefficients;counts[rep] <- fit$n;diagnostics[rep,] <- fit$diagnostics
}
N <- nrow(d);V <- cov(bs);means <- colMeans(bs);names <- c('_cons',predictors)
ci <- t(apply(bs,2,quantile,probs=c(.025,.975),type=7,names=FALSE))
fits <- data.frame(replicate=seq_len(B),sample_n=counts,diagnostics,bs,check.names=FALSE)
colnames(fits) <- c('replicate','sample_n',names(base$diagnostics),'intercept',predictors)
summary <- do.call(rbind,lapply(seq_len(7),function(i)do.call(rbind,lapply(seq_len(7),function(j)
  data.frame(parameter_index=i,covariance_index=j,parameter=names[i],covariance_parameter=names[j],
      estimate=base$coefficients[i],bootstrap_mean=means[i],bias=means[i]-base$coefficients[i],
      standard_error=sqrt(V[i,i]),ci_lower=ci[i,1],ci_upper=ci[i,2],covariance=V[i,j],
      replications=B,communes_per_draw=G,base_seed=10001,
      rng='xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1',
      method='whole_commune_pairs_refit_equal_group_re_gls',interval_method='type7_percentile_95')))))
d$re_xb <- base$coefficients[1]+drop(X%*%base$coefficients[-1])
d$re_residual <- d$lnexp12m-d$re_xb
stopifnot(nrow(fits)==500,nrow(summary)==49,all(is.finite(as.matrix(bs))),all(is.finite(V)))
if(!dir.create(a[3],recursive=TRUE)) stop('cannot create reference directory')
write_precise <- function(data,path) {
  numeric_columns <- vapply(data,is.numeric,logical(1))
  data[numeric_columns] <- lapply(data[numeric_columns],function(x)ifelse(is.na(x),NA_character_,sprintf('%.17g',x)))
  write.csv(data,path,row.names=FALSE,na='')
}
write_precise(source,file.path(a[3],'input.csv'))
write_precise(d,file.path(a[3],'predictions.csv'))
write_precise(fits,file.path(a[3],'fits.csv'))
write_precise(as.data.frame(as.list(base$diagnostics)),file.path(a[3],'baseline.csv'))
write_precise(summary,file.path(a[3],'summary.csv'))
write_precise(data.frame(commune=ids,observations=sizes),file.path(a[3],'groups.csv'))
cat(R.version.string,'N',N,'G',G,'B',B,'intercept',base$coefficients[1],
    'SE',sqrt(diag(V)),'raw u range',range(diagnostics[,2]),'replicate N range',range(counts),'\n')
