# Independent base-R logistic fit and fixed range/decile diagnostics.
args <- commandArgs(trailingOnly=TRUE)
if(length(args)!=2) stop('usage: Rscript overlap_reference.R nswpsid.da1 output-directory')
if(dir.exists(args[2])) stop('output directory exists')
x <- read.table(args[1],col.names=c('treat','age','educ','black','hisp','marr','re74','re75','re78','u74','u75'))
stopifnot(nrow(x)==2675,sum(x$treat)==185,all(x$treat %in% c(0,1)))
x <- transform(x,age_s=age/10,age_s2=(age/10)^2,educ_s=educ/10,educ_s2=(educ/10)^2,
  nodegree=as.numeric(educ<12),re74_s=re74/10000,re75_s=re75/10000,
  re74_s2=(re74/10000)^2,re75_s2=(re75/10000)^2,u74black=as.numeric(re74==0)*black)
fit <- glm(treat~age_s+age_s2+educ_s+educ_s2+marr+nodegree+black+hisp+re74_s+re75_s+re74_s2+re75_s2+u74black,
  family=binomial(),data=x,control=glm.control(epsilon=1e-13,maxit=100))
stopifnot(fit$converged,fit$rank==ncol(model.matrix(fit)))
x$pscore <- predict(fit,type='response')
lower <- max(min(x$pscore[x$treat==0]),min(x$pscore[x$treat==1]))
upper <- min(max(x$pscore[x$treat==0]),max(x$pscore[x$treat==1]))
stopifnot(lower<upper)
x$in_overlap <- x$pscore>=lower & x$pscore<=upper
x$stratum <- ifelse(x$in_overlap,ceiling(10*x$pscore),NA_integer_)
X <- model.matrix(fit); covariance <- solve(crossprod(X*sqrt(x$pscore*(1-x$pscore))))
se <- sqrt(diag(covariance)); stat <- coef(fit)/se
coef <- data.frame(parameter=names(coef(fit)),value=coef(fit),se=se,statistic=stat,
 p_value=2*pnorm(-abs(stat)),lower=coef(fit)-qnorm(.975)*se,upper=coef(fit)+qnorm(.975)*se,row.names=NULL)
k <- ncol(X)
cov <- data.frame(row=rep(colnames(X),each=k),column=rep(colnames(X),k),value=as.vector(t(covariance)))
dir.create(args[2])
write.csv(coef,file.path(args[2],'coefficients.csv'),row.names=FALSE)
write.csv(cov,file.path(args[2],'covariances.csv'),row.names=FALSE)
write.csv(data.frame(source_row=seq_len(nrow(x)),x,lower_bound=lower,upper_bound=upper),file.path(args[2],'scores.csv'),row.names=FALSE,na='')
