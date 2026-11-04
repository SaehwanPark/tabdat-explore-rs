# Independent base-R QR construction of the author's intercept-including Shea diagnostic.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=2) stop('usage: Rscript shea_reference.R base-reference/design.csv output-directory')
d <- read.csv(args[1],check.names=FALSE)
stopifnot(nrow(d)==4256,length(unique(d$id))==532)
xnames <- c('dlnwg','dkids','dageh','dagesq','ddisab')
znames <- c('kidsl1','agehl1','agesql1','disabl1','agehl2','kidsl2','agesql2','disabl2','lnwgl2')
fits <- list(); addfit <- function(y,x) {
 design <- cbind(intercept=1,as.matrix(x)); q <- qr(design,tol=1e-12)
 stopifnot(q$rank==ncol(design)); b <- drop(qr.coef(q,y))
 fitted <- drop(design%*%b); residual <- drop(y-fitted)
 inv <- chol2inv(qr.R(q)); inv <- inv[order(q$pivot),order(q$pivot)]
 covariance <- inv*sum(residual^2)/(length(y)-length(b))
 list(coefficients=b,fitted=fitted,residual=residual,covariance=covariance,
      r_squared=1-sum(residual^2)/sum((y-mean(y))^2),df=length(y)-length(b))
}
fits[[1]] <- addfit(d$dlnwg,d[,xnames[-1]])
d$wage_residual <- fits[[1]]$residual
for (j in seq_along(xnames)) {
 fits[[j+1]] <- addfit(d[[xnames[j]]],d[,znames])
 d[[paste0(xnames[j],'_hat')]] <- fits[[j+1]]$fitted
}
fitted_names <- paste0(xnames,'_hat')
fits[[7]] <- addfit(d$dlnwg_hat,d[,fitted_names[-1]])
d$projected_wage_residual <- fits[[7]]$residual
rho <- cor(d$wage_residual,d$projected_wage_residual)
d$panel_iv_fitted <- NULL; d$panel_iv_residual <- NULL
if (dir.exists(args[2])) stop('output directory exists')
dir.create(args[2]); write.csv(d,file.path(args[2],'design.csv'),row.names=FALSE)
coefficients <- do.call(rbind,lapply(seq_along(fits),function(i) {
 f <- fits[[i]]; se <- sqrt(diag(f$covariance))
 data.frame(fit=i,name=names(f$coefficients),value=unname(f$coefficients),
            se=se,p_value=2*pt(-abs(f$coefficients/se),f$df))
}))
write.csv(coefficients,file.path(args[2],'coefficients.csv'),row.names=FALSE)
for (i in seq_along(fits)) write.csv(fits[[i]]$covariance,
 file.path(args[2],paste0('covariance_',i,'.csv')),row.names=FALSE)
write.csv(data.frame(fit=seq_along(fits),r_squared=sapply(fits,function(f) f$r_squared)),
 file.path(args[2],'fit.csv'),row.names=FALSE)
write.csv(data.frame(n=nrow(d),groups=length(unique(d$id)),rho=rho,shea=rho^2),
 file.path(args[2],'diagnostic.csv'),row.names=FALSE)
