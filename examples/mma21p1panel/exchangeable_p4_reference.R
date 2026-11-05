#!/usr/bin/env Rscript
# Derive the P4 report from the independently fitted R gee/dense-matrix reference.
args <- commandArgs(trailingOnly=TRUE)
if(length(args)!=2) stop('usage: Rscript exchangeable_p4_reference.R gaussian_pa_reference_directory fresh_output_directory')
stopifnot(!file.exists(args[2]))
f <- read.csv(file.path(args[1],'fit.csv'),check.names=FALSE)
stopifnot(nrow(f)==1,f$people>=3,f$observations==10*f$people,f$years_per_person==10)
g <- f$people; mult <- g/(g-1)
W <- matrix(c(f$working_covariance_aa,f$working_covariance_ab,f$working_covariance_ab,f$working_covariance_bb),2,2)
C <- matrix(c(f$unit_covariance_aa,f$unit_covariance_ab,f$unit_covariance_ab,f$unit_covariance_bb),2,2)
V <- mult*C;b <- c(f$intercept,f$slope)
stopifnot(all(is.finite(c(W,C,V,b))),all(diag(W)>0),all(diag(C)>128*.Machine$double.eps*diag(W)))
fit <- data.frame(observations=f$observations,people=g,years_per_person=10,intercept=b[1],slope=b[2],
 scale=f$scale,working_correlation=f$working_correlation,working_covariance_aa=W[1,1],working_covariance_ab=W[1,2],
 working_covariance_bb=W[2,2],raw_unit_covariance_aa=C[1,1],raw_unit_covariance_ab=C[1,2],raw_unit_covariance_bb=C[2,2],
 unit_multiplier=mult,adjusted_unit_covariance_aa=V[1,1],adjusted_unit_covariance_ab=V[1,2],adjusted_unit_covariance_bb=V[2,2],
 intercept_step=f$intercept_step,slope_step=f$slope_step)
coefficients <- do.call(rbind,lapply(c('unit_sandwich_g_adjusted','working_model'),function(method) {
 vc <- if(method=='working_model') W else V;se <- sqrt(diag(vc));z <- b/se
 data.frame(covariance=method,parameter=c('_cons','lnwg'),estimate=b,standard_error=se,z=z,
  p_value=2*pnorm(-abs(z)),normal95_lower=b-qnorm(.975)*se,normal95_upper=b+qnorm(.975)*se,
  slope_wald_chi2_1=c(NA,z[2]^2))
}))
coefficients <- coefficients[order(coefficients$covariance,coefficients$parameter),]
dir.create(args[2])
write_precise <- function(frame,name) {
 text <- as.data.frame(lapply(frame,function(x) if(is.numeric(x)) ifelse(is.na(x),'',sprintf('%.17g',x)) else x))
 write.table(text,file.path(args[2],name),sep=',',quote=FALSE,row.names=FALSE,col.names=TRUE,na='')
}
write_precise(fit,'fit.csv');write_precise(coefficients,'coefficients.csv')
cat(R.version.string,'N',f$observations,'G',g,'unit multiplier',mult,'\n')
