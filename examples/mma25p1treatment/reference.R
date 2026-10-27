# Independent base-R QR and HC1 reference for the author ASCII sample.
args <- commandArgs(trailingOnly=TRUE)
if (length(args) != 2) stop('usage: Rscript reference.R nswpsid.da1 output-directory')
if (dir.exists(args[2])) stop('output directory already exists')
x <- read.table(args[1], col.names=c('treat','age','educ','black','hisp','marr','re74','re75','re78','u74','u75'))
stopifnot(nrow(x)==2675, sum(x$treat)==185, all(x$treat %in% c(0,1)))
x$age_sq <- x$age^2
x$nodegree <- as.numeric(x$educ<12)
forms <- list(unadjusted=re78~treat, adjusted=re78~treat+age+age_sq+educ+nodegree+black+hisp+re74+re75)
dir.create(args[2])
coefficients <- covariances <- predictions <- list()
for (name in names(forms)) {
  fit <- lm(forms[[name]], data=x)
  X <- model.matrix(fit); residual <- residuals(fit); n <- nrow(X); p <- ncol(X)
  bread <- chol2inv(chol(crossprod(X)))
  cov <- n/(n-p)*bread %*% crossprod(X * residual) %*% bread
  se <- sqrt(diag(cov)); statistic <- coef(fit)/se
  coefficients[[name]] <- data.frame(method=name, parameter=names(coef(fit)), value=coef(fit), se=se,
    statistic=statistic, p_value=2*pt(-abs(statistic),n-p), lower=coef(fit)-qt(.975,n-p)*se,
    upper=coef(fit)+qt(.975,n-p)*se, row.names=NULL)
  covariances[[name]] <- data.frame(method=name, row=rep(colnames(X), each=p), column=rep(colnames(X),p), value=as.vector(t(cov)))
  predictions[[name]] <- fitted(fit)
}
write.csv(do.call(rbind,coefficients),file.path(args[2],'coefficients.csv'),row.names=FALSE)
write.csv(do.call(rbind,covariances),file.path(args[2],'covariances.csv'),row.names=FALSE)
write.csv(data.frame(source_row=seq_len(nrow(x)),x,unadjusted_mean=predictions$unadjusted,adjusted_mean=predictions$adjusted),file.path(args[2],'predictions.csv'),row.names=FALSE)
