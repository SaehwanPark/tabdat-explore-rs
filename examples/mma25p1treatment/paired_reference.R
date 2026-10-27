# Independent base-R QR and person-score CR1 covariance.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=2) stop('usage: Rscript paired_reference.R nswpsid.da1 output-directory')
if (dir.exists(args[2])) stop('output directory exists')
x <- read.table(args[1],col.names=c('treat','age','educ','black','hisp','marr','re74','re75','re78','u74','u75'))
stopifnot(nrow(x)==2675, sum(x$treat)==185, all(x$treat %in% c(0,1)))
d <- data.frame(source_row=rep(seq_len(nrow(x)),each=2),treat=rep(x$treat,each=2),
  post=rep(0:1,nrow(x)),earnings=as.vector(t(cbind(x$re75,x$re78))))
d$interaction <- d$treat*d$post
fits <- list(before_after=lm(earnings~post,data=d,subset=treat==1),
             diff_in_diff=lm(earnings~interaction+treat+post,data=d))
dir.create(args[2])
coefficients <- covariances <- list()
for (name in names(fits)) {
  fit <- fits[[name]]; X <- model.matrix(fit); residual <- residuals(fit)
  id <- if (name=='before_after') d$source_row[d$treat==1] else d$source_row
  n <- nrow(X); k <- ncol(X); g <- length(unique(id))
  bread <- chol2inv(chol(crossprod(X)))
  scores <- rowsum(X * residual,id,reorder=FALSE)
  covariance <- (g/(g-1))*((n-1)/(n-k))*bread %*% crossprod(scores) %*% bread
  se <- sqrt(diag(covariance)); stat <- coef(fit)/se; crit <- qt(.975,g-1)
  coefficients[[name]] <- data.frame(model=name,parameter=names(coef(fit)),value=coef(fit),se=se,
    statistic=stat,p_value=2*pt(-abs(stat),g-1),lower=coef(fit)-crit*se,
    upper=coef(fit)+crit*se,n=n,groups=g,row.names=NULL)
  covariances[[name]] <- data.frame(model=name,row=rep(colnames(X),each=k),
    column=rep(colnames(X),k),value=as.vector(t(covariance)))
}
d$fitted_earnings <- fitted(fits$diff_in_diff)
write.csv(do.call(rbind,coefficients),file.path(args[2],'coefficients.csv'),row.names=FALSE)
write.csv(do.call(rbind,covariances),file.path(args[2],'covariances.csv'),row.names=FALSE)
write.csv(d,file.path(args[2],'predictions.csv'),row.names=FALSE)
