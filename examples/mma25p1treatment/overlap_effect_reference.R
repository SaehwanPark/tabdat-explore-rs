# Independent base-R direct score-system inversion and numerical derivative.
args <- commandArgs(trailingOnly=TRUE)
if(length(args)!=2) stop('usage: Rscript overlap_effect_reference.R nswpsid.da1 output-directory')
if(dir.exists(args[2])) stop('output directory exists')
x <- read.table(args[1],col.names=c('treat','age','educ','black','hisp','marr','re74','re75','re78','u74','u75'))
stopifnot(nrow(x)==2675,sum(x$treat)==185,all(x$treat %in% c(0,1)))
x <- transform(x,age_s=age/10,age_s2=(age/10)^2,educ_s=educ/10,educ_s2=(educ/10)^2,
  nodegree=as.numeric(educ<12),re74_s=re74/10000,re75_s=re75/10000,
  re74_s2=(re74/10000)^2,re75_s2=(re75/10000)^2,u74black=as.numeric(re74==0)*black)
fit <- glm(treat~age_s+age_s2+educ_s+educ_s2+marr+nodegree+black+hisp+re74_s+re75_s+re74_s2+re75_s2+u74black,
  family=binomial(),data=x,control=glm.control(epsilon=1e-13,maxit=100))
stopifnot(fit$converged,fit$rank==ncol(model.matrix(fit)))
X <- model.matrix(fit);p <- predict(fit,type='response');t <- x$treat;y <- x$re78
w1 <- t*(1-p);w0 <- (1-t)*p;s1 <- sum(w1);s0 <- sum(w0)
mu1 <- sum(w1*y)/s1;mu0 <- sum(w0*y)/s0
h <- crossprod(X,X*(p*(1-p)))
g <- -as.vector(crossprod(X,p*(1-p)*(t*(y-mu1)/s1+(1-t)*(y-mu0)/s0)))
v <- solve(h,g)
overlap_effect <- function(b) { pp <- plogis(as.vector(X%*%b)); a <- t*(1-pp); c <- (1-t)*pp
  sum(a*y)/sum(a)-sum(c*y)/sum(c) }
b <- coef(fit);step <- 1e-5
numerical <- vapply(seq_along(b),function(i) {d <- rep(0,length(b));d[i] <- step
  (overlap_effect(b+d)-overlap_effect(b-d))/(2*step)},0.0)
stopifnot(max(abs(numerical-g)/(1+abs(g)))<1e-6)
multiplier <- as.vector(X%*%v)
influence <- w1*(y-mu1)/s1-w0*(y-mu0)/s0+multiplier*(t-p)
se <- sqrt(sum(influence^2));estimate <- mu1-mu0
stopifnot(is.finite(se),se>0,abs(sum(influence)-sum(multiplier*(t-p)))<1e-9)
summary <- data.frame(overlap_effect=estimate,sandwich_se=se,normal_95_lower=estimate-qnorm(.975)*se,
 normal_95_upper=estimate+qnorm(.975)*se,weighted_treated_mean=mu1,weighted_control_mean=mu0,
 treated_weight_ess=s1^2/sum(w1^2),control_weight_ess=s0^2/sum(w0^2),
 max_treated_weight=max(w1),max_control_weight=max(w0))
rows <- data.frame(source_row=seq_len(nrow(x)),treat=t,re78=y,pscore=p,overlap_weight=w1+w0,
 mu1=mu1,mu0=mu0,nuisance_multiplier=multiplier,influence=influence)
balance <- data.frame(treat=c(0,1),people=c(sum(t==0),sum(t==1)),weight_sum=c(s0,s1),
  rbind(colSums(X*w0)/s0,colSums(X*w1)/s1),check.names=FALSE)
balance <- balance[,colnames(balance)!='(Intercept)']
dir.create(args[2])
write.csv(summary,file.path(args[2],'summary.csv'),row.names=FALSE)
write.csv(rows,file.path(args[2],'rows.csv'),row.names=FALSE)
write.csv(balance,file.path(args[2],'balance.csv'),row.names=FALSE)
write.csv(data.frame(parameter=names(b),gradient=g,numerical_gradient=numerical,multiplier=v),file.path(args[2],'gradient.csv'),row.names=FALSE)
