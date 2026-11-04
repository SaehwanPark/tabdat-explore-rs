#!/usr/bin/env Rscript
# Development-only independent raw-copy QR, not native weighted sufficient statistics.
a <- commandArgs(trailingOnly=TRUE)
if(length(a)!=3) stop('usage: Rscript fe_level_bootstrap_reference.R MOM.dat native_draws.csv fresh_reference')
stopifnot(!file.exists(a[3]))
d <- read.table(a[1],col.names=c('lnhr','lnwg','kids','ageh','agesq','disab','id','year'))
stopifnot(nrow(d)>0,nrow(d)<=5320,all(is.finite(as.matrix(d))),!anyDuplicated(d[c('id','year')]),
 all(d$id==floor(d$id)),all(d$id>=1 & d$id<=532),all(d$year==floor(d$year)),all(d$year>=1979 & d$year<=1988),
 all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
d$source_row <- seq_len(nrow(d));ids <- sort(unique(d$id));G <- length(ids);N <- nrow(d)
stopifnot(G>=3,G<=532,all(table(d$id)==10))
s <- d[order(d$id,d$year),]; X <- matrix(s$lnwg,ncol=10,byrow=TRUE);Y <- matrix(s$lnhr,ncol=10,byrow=TRUE)
qr_fit <- function(indices,replicate) {
 # Each draw position is a fresh ten-row person, even when original IDs repeat.
 x <- X[indices,,drop=FALSE];y <- Y[indices,,drop=FALSE]
 mx <- rowMeans(x);my <- rowMeans(y);dx <- as.vector(t(x-mx));dy <- as.vector(t(y-my))
 stopifnot(sum(dx^2)>1e-12)
 model <- lm.fit(matrix(dx,ncol=1),dy);stopifnot(model$rank==1)
 b <- unname(model$coefficients[1]);alpha <- my-b*mx;intercept <- mean(alpha)
 values <- c(replicate=replicate,intercept=intercept,slope=b,sample_mean_lnwg=mean(x),sample_mean_lnhr=mean(y),
  residual_df=N-G-1,within_rss=sum(model$residuals^2),normalized_effect_mean=mean(alpha-intercept))
 stopifnot(all(is.finite(values)),abs(intercept)<=100,abs(b)<=100)
 values
}
base <- qr_fit(seq_len(G),0)
# Sum-to-zero dummy columns identify precisely the reported common level intercept.
Z <- cbind(1,s$lnwg,contr.sum(G)[rep(seq_len(G),each=10),,drop=FALSE])
dense <- lm.fit(Z,s$lnhr)
stopifnot(dense$rank==G+1,abs(dense$coefficients[1]-base['intercept'])<1e-9,
 abs(dense$coefficients[2]-base['slope'])<1e-9,abs(sum(dense$residuals^2)-base['within_rss'])<1e-9)
draws <- read.csv(a[2]);stopifnot(nrow(draws)==500*G,
 identical(as.integer(draws$replicate),rep(1:500,each=G)),
 identical(as.integer(draws$copy_id),rep(seq_len(G),500)),all(draws$step>=draws$copy_id),all(draws$base_seed==10001))
index <- match(draws$person_id,ids);stopifnot(!anyNA(index))
fits <- do.call(rbind,lapply(1:500,function(r) qr_fit(index[draws$replicate==r],r)))
A <- fits[,'intercept'];B <- fits[,'slope']
summary <- data.frame(replications=500,people_per_draw=G,estimate_intercept=base['intercept'],estimate_slope=base['slope'],
 mean_intercept=mean(A),mean_slope=mean(B),bias_intercept=mean(A)-base['intercept'],bias_slope=mean(B)-base['slope'],
 covariance_aa=var(A),covariance_ab=cov(A,B),covariance_bb=var(B),se_intercept=sd(A),se_slope=sd(B),
 lower_intercept=quantile(A,.025,names=FALSE,type=7),upper_intercept=quantile(A,.975,names=FALSE,type=7),
 lower_slope=quantile(B,.025,names=FALSE,type=7),upper_slope=quantile(B,.975,names=FALSE,type=7))
unit_alpha <- rowMeans(Y)-base['slope']*rowMeans(X);alpha <- unit_alpha[match(d$id,ids)]
d$fe_xb <- unname(base['intercept']+base['slope']*d$lnwg)
d$fe_xbu <- unname(alpha+base['slope']*d$lnwg);d$fe_u <- unname(alpha-base['intercept'])
d$fe_residual <- d$lnhr-d$fe_xbu
out <- a[3];dir.create(out)
write_precise <- function(frame,name) {
 text <- as.data.frame(lapply(frame,function(x) if(is.numeric(x)) sprintf('%.17g',x) else x))
 write.table(text,file.path(out,name),sep=',',quote=FALSE,row.names=FALSE,col.names=TRUE)
}
write_precise(as.data.frame(rbind(base,fits)),'refits.csv');write_precise(summary,'summary.csv')
write_precise(data.frame(replicate=draws$replicate,position=draws$copy_id,step=draws$step,index0=index-1),'draws.csv')
write_precise(d,'predictions.csv')
cat(R.version.string,'; raw-copy QR refits500; baseline sum-contrast dummy QR; G',G,'N',N,'\n')
