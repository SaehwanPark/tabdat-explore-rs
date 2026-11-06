# Development-only independent raw-copy Poisson QR refits; never reads native fits.
options(digits=17)
a <- commandArgs(trailingOnly=TRUE)
if (length(a)<3 || length(a)>4 || (length(a)==4 && !a[4]%in%c('--csv','--dta')))
  stop('usage: individual_pooled_bootstrap_reference.R source oracle-draws.csv output-directory [--csv | --dta]')
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
  tokens <- scan(a[1],what=character(),quiet=TRUE)
  stopifnot(length(tokens)==27766*12)
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
  all(d$commune>=1 & d$commune<=194 & d$commune==floor(d$commune)),
  all(vapply(d[predictors],function(v)all(abs(v)<=1e6),logical(1))))
X <- cbind(1,as.matrix(d[predictors]));y <- d$pharvis
ids <- unique(d$commune);G <- length(ids);B <- 500L;K <- ncol(X)
members <- lapply(ids,function(id)which(d$commune==id));sizes <- lengths(members)
stopifnot(G>=2,G<=194,max(sizes)<=206,K==11)
draws <- read.csv(a[2]);stopifnot(identical(names(draws),c('replicate','copy_id','cluster_index','commune')),
  nrow(draws)==B*G,identical(draws$replicate,rep(seq_len(B),each=G)),
  identical(draws$copy_id,rep(seq_len(G),times=B)),all(draws$cluster_index>=0 & draws$cluster_index<G),
  all(draws$commune==ids[draws$cluster_index+1L]))
refit <- function(index) {
  fit <- glm.fit(X[index,,drop=FALSE],y[index],family=poisson(),
    control=glm.control(epsilon=1e-13,maxit=100))
  stopifnot(fit$converged,fit$rank==K,all(is.finite(fit$coefficients)),all(is.finite(fit$fitted.values)))
  fit
}
base <- refit(seq_len(nrow(d)));bs <- matrix(NA_real_,nrow=B,ncol=K);counts <- integer(B)
for(r in seq_len(B)) {
  chosen <- draws$cluster_index[draws$replicate==r]+1L
  index <- unlist(members[chosen],use.names=FALSE)
  fit <- refit(index);bs[r,] <- fit$coefficients;counts[r] <- length(index)
}
names <- c('intercept',predictors);V <- cov(bs);means <- colMeans(bs)
ci <- t(apply(bs,2,quantile,probs=c(.025,.975),type=7,names=FALSE))
fits <- data.frame(replicate=seq_len(B),sample_n=counts,bs,check.names=FALSE)
colnames(fits) <- c('replicate','sample_n',names)
summary <- do.call(rbind,lapply(seq_len(K),function(i)do.call(rbind,lapply(seq_len(K),function(j)
  data.frame(parameter_index=i,covariance_index=j,parameter=names[i],covariance_parameter=names[j],
    estimate=base$coefficients[i],bootstrap_mean=means[i],bias=means[i]-base$coefficients[i],
    standard_error=sqrt(V[i,i]),ci_lower=ci[i,1],ci_upper=ci[i,2],covariance=V[i,j])))))
d$pooled_mu <- base$fitted.values;d$pooled_xb <- drop(X%*%base$coefficients)
stopifnot(nrow(fits)==B,nrow(summary)==K*K,all(is.finite(bs)),all(is.finite(V)))
if(!dir.create(a[3],recursive=TRUE)) stop('cannot create reference directory')
write_precise <- function(data,path) {
  numeric_columns <- vapply(data,is.numeric,logical(1))
  data[numeric_columns] <- lapply(data[numeric_columns],function(x)ifelse(is.na(x),NA_character_,sprintf('%.17g',x)))
  write.csv(data,path,row.names=FALSE,na='')
}
write_precise(source,file.path(a[3],'source.csv'))
write_precise(d,file.path(a[3],'predictions.csv'))
write_precise(fits,file.path(a[3],'fits.csv'))
write_precise(summary,file.path(a[3],'summary.csv'))
write_precise(data.frame(commune=ids,observations=sizes),file.path(a[3],'groups.csv'))
cat(R.version.string,'N',nrow(d),'G',G,'B',B,'replicate N range',range(counts),'\n')
