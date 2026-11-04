#!/usr/bin/env Rscript
# Legacy gee is independent of the native SQL; dense per-person matrices audit both covariances.
args <- commandArgs(trailingOnly=TRUE)
if(length(args)!=2) stop('usage: Rscript gaussian_pa_reference.R MOM.dat fresh_reference_directory')
stopifnot(requireNamespace('gee',quietly=TRUE),packageVersion('gee')==package_version('4.13.30'),!file.exists(args[2]))
d <- read.table(args[1],col.names=c('lnhr','lnwg','kids','ageh','agesq','disab','id','year'))
stopifnot(nrow(d)>0,nrow(d)<=5320,all(is.finite(as.matrix(d))),!anyDuplicated(d[c('id','year')]),
 all(d$id==floor(d$id)),all(d$id>=1 & d$id<=532),all(d$year==floor(d$year)),all(d$year>=1979 & d$year<=1988),
 all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
d$source_row <- seq_len(nrow(d)); ids <- sort(unique(d$id)); G <- length(ids); N <- nrow(d)
stopifnot(G>=3,G<=532,all(table(d$id)==10))
sorted <- d[order(d$id,d$year),]; X <- cbind(1,sorted$lnwg); y <- sorted$lnhr
blocks <- lapply(ids,function(id) which(sorted$id==id))
start <- unname(coef(lm(lnhr~lnwg,sorted)))
evaluate <- function(beta) {
 residual <- drop(y-X%*%beta)
 E <- matrix(residual,ncol=10,byrow=TRUE)
 rss <- sum(E^2); phi <- rss/N
 sw <- sum((E-rowMeans(E))^2); sb <- 10*sum(rowMeans(E)^2)
 stopifnot(all(is.finite(c(beta,sw,sb))),sw+sb>1e-12,
  sw>128*.Machine$double.eps*(sw+sb),sb>128*.Machine$double.eps*(sw+sb))
 rho <- sum(rowSums(E)^2-rowSums(E^2))/(9*rss)
 V <- phi*((1-rho)*diag(10)+rho*matrix(1,10,10)); IV <- solve(V)
 H <- matrix(0,2,2); xy <- numeric(2); scores <- matrix(NA_real_,G,2)
 for(i in seq_len(G)) {
  index <- blocks[[i]]; Z <- X[index,,drop=FALSE]
  H <- H+crossprod(Z,IV%*%Z); xy <- xy+drop(crossprod(Z,IV%*%y[index]))
  scores[i,] <- drop(crossprod(Z,IV%*%residual[index]))
 }
 W <- solve(H); influence <- scores%*%W
 list(phi=phi,rho=rho,W=W,C=crossprod(influence),scores=scores,influence=influence,
  next_beta=drop(solve(H,xy)),e=phi*(1-rho),v=phi*(1+9*rho))
}
# Preflight before legacy Cgee: singular correlation can otherwise stall the package.
beta <- start; converged <- FALSE
for(i in 1:100) {
 value <- evaluate(beta); next_beta <- value$next_beta
 if(max(abs(next_beta-beta)/(1+abs(beta)))<=1e-12) { beta <- next_beta;converged <- TRUE;break }
 beta <- next_beta
}
stopifnot(converged)
capture.output(model <- gee::gee(lnhr~lnwg,id=id,data=sorted,family=gaussian(),corstr='exchangeable',
 b=start,scale.fix=FALSE,v4.4compat=TRUE,tol=1e-13,maxiter=100,silent=TRUE))
stopifnot(model$error==0,model$iterations<100)
b <- unname(coef(model)); final <- evaluate(b)
stopifnot(max(abs(b-beta))<1e-9,max(abs(model$naive.variance-final$W))<1e-9,
 max(abs(model$robust.variance-final$C))<1e-9,abs(model$scale-final$phi)<1e-10,
 abs(model$working.correlation[1,2]-final$rho)<1e-10)
step <- drop(final$W%*%colSums(final$scores)); grandx <- mean(sorted$lnwg)
fit <- data.frame(observations=N,people=G,years_per_person=10,intercept=b[1],slope=b[2],covariate_mean=grandx,
 scale=final$phi,working_correlation=final$rho,within_eigenvariance=final$e,between_eigenvariance=final$v,
 working_covariance_aa=final$W[1,1],working_covariance_ab=final$W[1,2],working_covariance_bb=final$W[2,2],
 unit_covariance_aa=final$C[1,1],unit_covariance_ab=final$C[1,2],unit_covariance_bb=final$C[2,2],
 intercept_step=step[1],slope_step=step[2])
coefficients <- do.call(rbind,lapply(c('working_model','uncorrected_unit_sandwich'),function(method) {
 V <- if(method=='working_model') final$W else final$C
 se <- sqrt(diag(V));data.frame(covariance=method,parameter=c('_cons','lnwg'),estimate=b,standard_error=se,
  z=b/se,normal95_lower=b-qnorm(.975)*se,normal95_upper=b+qnorm(.975)*se)
}))
coefficients <- coefficients[order(coefficients$covariance,coefficients$parameter),]
units <- data.frame(id=ids,observations=10,score_intercept=final$scores[,1],
 score_centered_slope=final$scores[,2]-grandx*final$scores[,1],
 influence_intercept=final$influence[,1],influence_slope=final$influence[,2])
d$pols_mean <- start[1]+start[2]*d$lnwg;d$pols_residual <- d$lnhr-d$pols_mean
d$gee_mean <- b[1]+b[2]*d$lnwg;d$gee_residual <- d$lnhr-d$gee_mean
out <- args[2];dir.create(out)
write_precise <- function(frame,name) {
 text <- as.data.frame(lapply(frame,function(x) if(is.numeric(x)) sprintf('%.17g',x) else x))
 write.table(text,file.path(out,name),sep=',',quote=FALSE,row.names=FALSE,col.names=TRUE)
}
write_precise(fit,'fit.csv');write_precise(coefficients,'coefficients.csv')
write_precise(d,'predictions.csv');write_precise(units,'unit_scores.csv')
cat(R.version.string,'gee',as.character(packageVersion('gee')),'G',G,'N',N,'rho',final$rho,'scale',final$phi,'\n')
