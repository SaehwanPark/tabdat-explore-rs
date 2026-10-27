# Independent base-R pair matrices, estimating-equation sandwich and gradient.
args <- commandArgs(trailingOnly=TRUE)
if(length(args)!=2) stop('usage: Rscript kernel_matching_reference.R nswpsid.da1 output-directory')
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
it <- which(t==1);ic <- which(t==0);n1 <- length(it);h <- 0.06
pair <- outer(p[it],p[ic],'-'); kernel <- exp(-0.5*(pair/h)^2); q <- kernel/rowSums(kernel)
matched <- as.vector(q%*%y[ic]);contrast <- y[it]-matched;estimate <- mean(contrast)
residual <- outer(matched,y[ic],function(m,y) y-m)
direct <- numeric(nrow(x));direct[it] <- (contrast-estimate)/n1
direct[ic] <- -colSums(q*residual)/n1
score_numerator <- numeric(nrow(x));weighted <- q*residual*pair/(h*h*n1)
score_numerator[it] <- rowSums(weighted)*p[it]*(1-p[it])
score_numerator[ic] <- -colSums(weighted)*p[ic]*(1-p[ic])
H <- crossprod(X,X*(p*(1-p)));g <- as.vector(crossprod(X,score_numerator));v <- solve(H,g)
matching_contrast <- function(b) { pp <- plogis(as.vector(X%*%b));delta <- outer(pp[it],pp[ic],'-')
  K <- exp(-0.5*(delta/h)^2);mean(y[it]-as.vector((K/rowSums(K))%*%y[ic])) }
b <- coef(fit);step <- 1e-6
numerical <- vapply(seq_along(b),function(i) {d <- rep(0,length(b));d[i] <- step
  (matching_contrast(b+d)-matching_contrast(b-d))/(2*step)},0.0)
stopifnot(max(abs(numerical-g)/(1+abs(g)))<1e-6)
multiplier <- as.vector(X%*%v);influence <- direct+multiplier*(t-p);se <- sqrt(sum(influence^2))
stopifnot(is.finite(se),se>0,abs(sum(influence)-sum(multiplier*(t-p)))<1e-9)
match_ess <- 1/rowSums(q*q);nearest <- apply(abs(pair),1,min);reuse <- colSums(q)
summary <- data.frame(kernel_contrast=estimate,sandwich_se=se,normal_95_lower=estimate-qnorm(.975)*se,
 normal_95_upper=estimate+qnorm(.975)*se,min_match_ess=min(match_ess),mean_match_ess=mean(match_ess),
 max_nearest_distance=max(nearest),max_control_reuse=max(reuse))
matched_all <- nearest_all <- ess_all <- reuse_all <- rep(NA_real_,nrow(x))
matched_all[it] <- matched;nearest_all[it] <- nearest;ess_all[it] <- match_ess;reuse_all[ic] <- reuse
records <- data.frame(source_row=seq_len(nrow(x)),treat=t,re78=y,pscore=p,matched_mean=matched_all,
 nearest_distance=nearest_all,match_ess=ess_all,reuse_weight=reuse_all,direct_influence=direct,
 score_numerator=score_numerator,nuisance_multiplier=multiplier,influence=influence)
dir.create(args[2])
write.csv(summary,file.path(args[2],'summary.csv'),row.names=FALSE)
write.csv(records,file.path(args[2],'rows.csv'),row.names=FALSE,na='')
write.csv(data.frame(parameter=names(b),gradient=g,numerical_gradient=numerical,multiplier=v),file.path(args[2],'gradient.csv'),row.names=FALSE)
