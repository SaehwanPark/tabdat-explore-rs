# Independent base-R joint estimating-equation sandwich and derivative check.
args <- commandArgs(trailingOnly=TRUE)
if(length(args)!=2) stop('usage: Rscript ipw_att_reference.R nswpsid.da1 output-directory')
if(dir.exists(args[2])) stop('output directory exists')
x <- read.table(args[1],col.names=c('treat','age','educ','black','hisp','marr','re74','re75','re78','u74','u75'))
stopifnot(nrow(x)==2675,sum(x$treat)==185,all(x$treat %in% c(0,1)))
x <- transform(x,age_s=age/10,age_s2=(age/10)^2,educ_s=educ/10,educ_s2=(educ/10)^2,
  nodegree=as.numeric(educ<12),re74_s=re74/10000,re75_s=re75/10000,
  re74_s2=(re74/10000)^2,re75_s2=(re75/10000)^2,u74black=as.numeric(re74==0)*black)
fit <- glm(treat~age_s+age_s2+educ_s+educ_s2+marr+nodegree+black+hisp+re74_s+re75_s+re74_s2+re75_s2+u74black,
  family=binomial(),data=x,control=glm.control(epsilon=1e-13,maxit=100))
stopifnot(fit$converged,fit$rank==ncol(model.matrix(fit)))
X <- model.matrix(fit); p <- predict(fit,type='response'); t <- x$treat; y <- x$re78
w <- (1-t)*p/(1-p); sw <- sum(w); n1 <- sum(t)
mu1 <- sum(t*y)/n1; mu0 <- sum(w*y)/sw
h <- crossprod(X,X*(p*(1-p)))
g <- as.vector(crossprod(X,w*(y-mu0)))/sw
v <- solve(h,g)
# Central differences independently check the normalized weighted-mean gradient.
weighted_mean <- function(b) { odds <- exp(as.vector(X %*% b)); ww <- (1-t)*odds; sum(ww*y)/sum(ww) }
b <- coef(fit); step <- 1e-5
numerical <- vapply(seq_along(b),function(i) {d <- rep(0,length(b));d[i] <- step
  (weighted_mean(b+d)-weighted_mean(b-d))/(2*step)},0.0)
stopifnot(max(abs(numerical-g)/(1+abs(g)))<1e-6)
multiplier <- as.vector(X %*% v)
influence <- t*(y-mu1)/n1-w*(y-mu0)/sw-multiplier*(t-p)
variance <- sum(influence^2); se <- sqrt(variance); estimate <- mu1-mu0
stopifnot(is.finite(se),se>0,abs(sum(influence))<1e-9)
summary <- data.frame(att=estimate,sandwich_se=se,normal_95_lower=estimate-qnorm(.975)*se,
 normal_95_upper=estimate+qnorm(.975)*se,treated_mean=mu1,weighted_control_mean=mu0,
 control_weight_ess=sw^2/sum(w^2),max_control_weight=max(w),min_score=min(p),max_score=max(p))
rows <- data.frame(source_row=seq_len(nrow(x)),treat=t,re78=y,pscore=p,control_weight=w,
 treated_mean=mu1,weighted_control_mean=mu0,nuisance_multiplier=multiplier,influence=influence)
dir.create(args[2])
write.csv(summary,file.path(args[2],'summary.csv'),row.names=FALSE)
write.csv(rows,file.path(args[2],'rows.csv'),row.names=FALSE)
write.csv(data.frame(parameter=names(b),gradient=g,numerical_gradient=numerical,multiplier=v),file.path(args[2],'gradient.csv'),row.names=FALSE)
