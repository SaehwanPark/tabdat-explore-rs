#!/usr/bin/env Rscript
# Independent per-copy panel fits using plm, plus direct transformed-design QR.
args <- commandArgs(trailingOnly=TRUE)
if(length(args)!=2) stop('usage: Rscript random_effects_gls_refit_bootstrap_reference.R MOM.dat fresh_output_directory')
self <- sub('^--file=','',commandArgs(FALSE)[grepl('^--file=',commandArgs(FALSE))])
repo <- normalizePath(file.path(dirname(self),'../..'))
out <- args[2]
if(file.exists(out)) stop('refusing existing reference directory')
d <- read.table(args[1],col.names=c('lnhr','lnwg','kids','ageh','agesq','disab','id','year'))
stopifnot(nrow(d)>0,nrow(d)<=5320,all(is.finite(as.matrix(d))),!anyDuplicated(d[c('id','year')]),
 all(d$id==floor(d$id)),all(d$id>=1 & d$id<=532),all(d$year==floor(d$year)),all(d$year>=1979 & d$year<=1988),
 all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
d$source_row <- seq_len(nrow(d)); ids <- sort(unique(d$id)); G <- length(ids)
stopifnot(G>=3,G<=532,all(table(d$id)==10))
blocks <- lapply(ids,function(id) d[d$id==id,][order(d$year[d$id==id]),])
dir.create(out)
status <- system2('python3',c(shQuote(file.path(repo,'examples/mma23p1pannonlin/conditional_bootstrap_draws.py')),
 '10001',G,'500',G,shQuote(file.path(out,'draws.csv'))))
stopifnot(status==0)
draws <- read.csv(file.path(out,'draws.csv'))
fit <- function(which) {
 z <- do.call(rbind,blocks[which]); groups <- length(which); N <- nrow(z)
 z$copy_id <- rep(seq_len(groups),each=10)
 re <- plm::plm(lnhr~lnwg,data=z,index=c('copy_id','year'),model='random',random.method='swar')
 W <- plm::plm(lnhr~lnwg,data=z,index=c('copy_id','year'),model='within')
 means <- aggregate(cbind(lnhr,lnwg)~copy_id,z,mean)
 B <- lm(lnhr~lnwg,means)
 e <- sum(resid(W)^2)/(N-groups-1)
 rawu <- sum(resid(B)^2)/(groups-2)-e/10
 u <- max(0,rawu); theta <- 1-sqrt(e/(e+10*u))
 stopifnot(max(abs(c(e,u)-unname(re$ercomp$sigma2)))<1e-10,
  abs(theta-unname(re$ercomp$theta))<1e-10)
 ym <- ave(z$lnhr,z$copy_id); xm <- ave(z$lnwg,z$copy_id)
 X <- cbind(rep(1-theta,N),z$lnwg-theta*xm)
 qrfit <- lm.fit(X,z$lnhr-theta*ym)
 b <- unname(coef(re))
 stopifnot(qrfit$rank==2,max(abs(b-qrfit$coefficients))<1e-10,
  all(is.finite(c(b,e,u,rawu,theta))),e>0,theta>=0,theta<1)
 c(intercept=b[1],slope=b[2],sigma_e_squared=e,sigma_u_squared=u,
  raw_sigma_u_squared=rawu,theta=theta,within_slope=unname(coef(W)[1]),
  between_slope=unname(coef(B)[2]),transformed_rss=sum(qrfit$residuals^2))
}
results <- vector('list',501)
for(rep in 0:500) {
 which <- if(rep==0) seq_len(G) else draws$index0[draws$replicate==rep]+1
 results[[rep+1]] <- c(replicate=rep,fit(which))
}
r <- as.data.frame(do.call(rbind,results)); original <- r[1,]; b <- r[-1,]
V <- cov(b[c('intercept','slope')]); ci <- sapply(b[c('intercept','slope')],quantile,c(.025,.975),type=7)
summary <- data.frame(replications=500,people_per_draw=G,
 estimate_intercept=original$intercept,estimate_slope=original$slope,
 mean_intercept=mean(b$intercept),mean_slope=mean(b$slope),
 bias_intercept=mean(b$intercept)-original$intercept,bias_slope=mean(b$slope)-original$slope,
 covariance_aa=V[1,1],covariance_ab=V[1,2],covariance_bb=V[2,2],
 se_intercept=sqrt(V[1,1]),se_slope=sqrt(V[2,2]),
 lower_intercept=ci[1,1],upper_intercept=ci[2,1],lower_slope=ci[1,2],upper_slope=ci[2,2])
d$regls_xb <- original$intercept+original$slope*d$lnwg
d$regls_residual <- d$lnhr-d$regls_xb
write_precise <- function(frame,path) {
 text <- as.data.frame(lapply(frame,function(x) sprintf('%.17g',x)))
 write.table(text,path,sep=',',row.names=FALSE,col.names=TRUE,quote=FALSE)
}
write_precise(r,file.path(out,'refits.csv'))
write_precise(summary,file.path(out,'summary.csv'))
write_precise(d,file.path(out,'predictions.csv'))
cat(R.version.string,'plm',as.character(packageVersion('plm')),'G',G,'B500; 501 Swamy–Arora/QR refits; SE',sqrt(diag(V)),'\n')
