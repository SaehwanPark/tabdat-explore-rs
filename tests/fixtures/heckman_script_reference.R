# Development-only closed-form control for the ACTUAL script boundary.
# Not book data:24 rows,3 groups, selected counts2/4/6 of8.
# Usage: Rscript heckman_script_reference.R OUTPUT_DIRECTORY PINNED_heckitVcov.R
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2,unname(tools::md5sum(args[2]))=='40b68e067652a7419fd68541a197eeb6')
reference <- new.env(parent=baseenv());sys.source(args[2],envir=reference)
x <- rep(c(-1,0,1),each=8);d <- unlist(lapply(c(2,4,6),function(n)c(rep(1,n),rep(0,8-n))))
a <- qnorm(.75);gamma <- c(0,a);index <- a*x;p <- pnorm(index)
mills <- dnorm(index)/p;delta <- mills*(mills+index);selected <- d==1
alpha <- .5;residual_scale <- sqrt(1-alpha^2*mean(delta[selected]))
e <- rep(c(-residual_scale,residual_scale),12)
y <- 3+2*x+alpha*mills+e
z <- cbind(1,x);w <- cbind(1,x,mills)[selected,,drop=FALSE]
outcome <- lm.fit(w,y[selected]);beta <- unname(outcome$coefficients)
stopifnot(max(abs(beta-c(3,2,.5)))<1e-11)
# Exact within-group event proportions give zero probit score at gamma.
score <- crossprod(z,ifelse(selected,dnorm(index)/p,-dnorm(index)/(1-p)))
stopifnot(max(abs(score))<1e-12)
curvature <- ifelse(selected,mills*(mills+index),(dnorm(index)/(1-p))*(dnorm(index)/(1-p)-index))
selection_covariance <- solve(crossprod(z,curvature*z))
sse <- sum(outcome$residuals^2);sigma <- sqrt(sse/12+beta[3]^2*mean(delta[selected]));rho <- beta[3]/sigma
stopifnot(abs(sigma-1)<1e-12,abs(rho-.5)<1e-12)
covariance <- reference$heckitVcov(w,z[selected,,drop=FALSE],selection_covariance,rho,delta[selected],sigma)
invisible(chol(covariance))
rows <- list()
emit <- function(quantity,value,i=0,j=0)rows[[length(rows)+1L]] <<- data.frame(quantity=quantity,i=i,j=j,value=as.numeric(value))
inference <- function(stage,b,v) {
 for(i in seq_along(b)) {
  se <- sqrt(v[i,i]);statistic <- b[i]/se
  fields <- c(value=unname(b[i]),standard_error=se,statistic=unname(statistic),p_value=unname(2*pnorm(abs(statistic),lower.tail=FALSE)),confidence_lower=unname(b[i]-qnorm(.975)*se),confidence_upper=unname(b[i]+qnorm(.975)*se))
  for(name in names(fields))emit(paste0(stage,'_',name),fields[name],i)
  for(j in seq_along(b))emit(paste0(stage,'_covariance'),v[i,j],i,j)
 }
}
inference('selection',gamma,selection_covariance);inference('outcome',beta,covariance)
values <- c(n=24,selected_count=12,sigma=sigma,rho=rho,outcome_sse=sse,selection_log_likelihood=sum(ifelse(selected,log(p),log1p(-p))))
for(name in names(values))emit(name,values[name])
write.csv(data.frame(source_row=1:24,e=e,lnwage=x,ystar=y,ytrunc=ifelse(selected,y,NA),ycens=pmax(y,0),dy=d),file.path(args[1],'heckman_script_data.csv'),row.names=FALSE,na='')
latent <- 3+2*x
write.csv(data.frame(source_row=1:24,latent=latent,selection_probability=p,inverse_mills=mills,conditional_selected_mean=latent+alpha*mills,observed_contribution_mean=p*latent+alpha*dnorm(index)),file.path(args[1],'heckman_script_predictions.csv'),row.names=FALSE)
write.csv(do.call(rbind,rows),file.path(args[1],'heckman_script_reference.csv'),row.names=FALSE)
cat(R.version.string,'; closed-form probit/OLS moments and pinned sampleSelection1.2-16 covariance; beta=(3,2),alpha=.5,sigma=1,rho=.5\n')
