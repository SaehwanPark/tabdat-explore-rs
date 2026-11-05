#!/usr/bin/env Rscript
# Base-R QR and group scores, independent of the native SQL composition.
args <- commandArgs(trailingOnly=TRUE)
if(length(args)!=2) stop('usage: Rscript independent_gee_reference.R MOM.csv fresh_reference_directory')
stopifnot(!file.exists(args[2]))
d <- read.csv(args[1],check.names=FALSE)
required <- c('source_row','lnhr','lnwg','kids','ageh','agesq','disab','id','year')
stopifnot(identical(names(d),required),nrow(d)>0,nrow(d)<=5320,
 all(is.finite(as.matrix(d))),!anyDuplicated(d[c('id','year')]),!anyDuplicated(d$source_row),
 all(d$id==floor(d$id) & d$id>=1 & d$id<=532),all(d$year==floor(d$year) & d$year>=1979 & d$year<=1988),
 all(d$source_row==floor(d$source_row) & d$source_row>=1 & d$source_row<=5320),
 all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
d <- d[order(d$source_row),]; N <- nrow(d);ids <- sort(unique(d$id));G <- length(ids)
stopifnot(G>=3,G<=532,all(table(d$id)==10))
X <- cbind(1,d$lnwg); y <- d$lnhr
fit <- lm.fit(X,y);stopifnot(fit$rank==2,all(is.finite(fit$coefficients)))
b <- unname(fit$coefficients);e <- y-drop(X%*%b);rss <- sum(e^2);sxx <- sum((d$lnwg-mean(d$lnwg))^2)
stopifnot(rss>1e-12,sxx>1e-12,all(abs(b)<=100))
# QR inversion provides the independent reference bread, not the centered SQL formula.
bread <- chol2inv(qr.R(qr(X)));phi <- rss/N;working <- phi*bread
scores <- rowsum(X*e,d$id,reorder=TRUE);influence <- scores%*%bread
unit <- G/(G-1)*crossprod(influence)
stopifnot(all(is.finite(c(working,unit))),all(diag(unit)>0))
step <- drop(bread%*%colSums(scores))
fit_out <- data.frame(observations=N,people=G,years_per_person=10,intercept=b[1],slope=b[2],
 covariate_mean=mean(d$lnwg),rss=rss,wage_centered_ss=sxx,scale=phi,working_correlation=0,
 working_covariance_aa=working[1,1],working_covariance_ab=working[1,2],working_covariance_bb=working[2,2],
 unit_covariance_aa=unit[1,1],unit_covariance_ab=unit[1,2],unit_covariance_bb=unit[2,2],
 intercept_step=step[1],slope_step=step[2])
coefs <- do.call(rbind,lapply(c('unit_sandwich_g_adjusted','working_model'),function(method) {
 V <- if(method=='working_model') working else unit;se <- sqrt(diag(V));z <- b/se
 data.frame(covariance=method,parameter=c('_cons','lnwg'),estimate=b,standard_error=se,z=z,
  p_value=2*pnorm(-abs(z)),normal95_lower=b-qnorm(.975)*se,normal95_upper=b+qnorm(.975)*se,
  slope_wald_chi2_1=c(NA,z[2]^2))
}))
coefs <- coefs[order(coefs$covariance,coefs$parameter),]
units <- data.frame(id=ids,observations=10,score_intercept=scores[,1],
 score_centered_slope=scores[,2]-mean(d$lnwg)*scores[,1],
 influence_intercept=influence[,1],influence_slope=influence[,2])
d$pols_mean <- drop(X%*%b);d$pols_residual <- e;d$gee_mean <- drop(X%*%b);d$gee_residual <- e
out <- args[2];dir.create(out)
write_precise <- function(frame,name) {
 text <- as.data.frame(lapply(frame,function(x) if(is.numeric(x)) ifelse(is.na(x),'',sprintf('%.17g',x)) else x))
 write.table(text,file.path(out,name),sep=',',quote=FALSE,row.names=FALSE,col.names=TRUE,na='')
}
write_precise(fit_out,'fit.csv');write_precise(coefs,'coefficients.csv')
write_precise(d,'predictions.csv');write_precise(units,'unit_scores.csv')
cat(R.version.string,'N',N,'G',G,'scale',phi,'\n')
