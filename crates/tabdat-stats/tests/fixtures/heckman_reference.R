# Development-only deterministic synthetic fixtures. Not replacement book data.
# Usage: Rscript heckman_reference.R OUTPUT_DIRECTORY PINNED_heckitVcov.R
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2,unname(tools::md5sum(args[2]))=='40b68e067652a7419fd68541a197eeb6')
reference <- new.env(parent=baseenv());sys.source(args[2],envir=reference)
quantities <- list();data_rows <- list();predictions <- list()
emit <- function(case,quantity,value,i=0,j=0)
  quantities[[length(quantities)+1L]] <<- data.frame(case=case,quantity=quantity,i=i,j=j,value=as.numeric(value))
inference <- function(case,stage,b,v) {
  for(i in seq_along(b)) {
    se <- sqrt(v[i,i]);statistic <- b[i]/se
    fields <- c(value=unname(b[i]),standard_error=se,statistic=unname(statistic),
      p_value=unname(2*pnorm(abs(statistic),lower.tail=FALSE)),
      confidence_lower=unname(b[i]-qnorm(.975)*se),confidence_upper=unname(b[i]+qnorm(.975)*se))
    for(name in names(fields))emit(case,paste0(stage,'_',name),fields[name],i)
    for(j in seq_along(b))emit(case,paste0(stage,'_covariance'),v[i,j],i,j)
  }
}
fit_selection <- function(z,d) {
  start <- glm.fit(z,d,family=binomial(link='probit'),control=glm.control(epsilon=1e-12,maxit=100))
  q <- ncol(z);g <- paste0('g',1:q);x <- paste0('x',1:q)
  linear <- paste(paste0(g,'*',x),collapse='+')
  expression <- deriv3(as.formula(paste0('~s*log(pnorm(',linear,'))+(1-s)*log(pnorm(-(',linear,')))')),
    g,function.arg=c(g,x,'s'))
  evaluate <- function(theta) {
    v <- do.call(expression,c(as.list(theta),unname(as.list(as.data.frame(z))),list(d)))
    list(ll=sum(v),score=colSums(attr(v,'gradient')),information=-apply(attr(v,'hessian'),c(2,3),sum))
  }
  theta <- unname(start$coefficients)
  for(iteration in 1:20) {
    e <- evaluate(theta);invisible(chol(e$information));step <- solve(e$information,e$score)
    if(max(abs(e$score)*pmax(1,abs(theta)))/length(d)<1e-12 && max(abs(step)/pmax(1,abs(theta)))<1e-12)
      return(list(theta=unname(theta),covariance=solve(e$information),ll=e$ll))
    theta <- unname(theta+step)
  }
  stop('probit observed derivative certificate failed')
}
i <- 1:360
normal <- function(multiplier)qnorm(((i*multiplier)%%359+.5)/359)
x <- normal(37);z <- normal(71);v <- normal(53);u <- normal(101)
for(case in c('exclusion','no_constant')) {
  constant <- case=='exclusion'
  d <- as.numeric((if(constant)-.2 else 0)+.4*x+.7*z+v>0)
  y <- (if(constant)2 else 0)+.8*x+.5*v+sqrt(.75)*u
  selection_design <- if(constant)cbind(1,x,z)else cbind(x,z)
  outcome_design <- if(constant)cbind(1,x)else cbind(x)
  first <- fit_selection(selection_design,d)
  t <- drop(selection_design%*%first$theta);p <- pnorm(t)
  mills <- exp(dnorm(t,log=TRUE)-pnorm(t,log.p=TRUE));delta <- mills*(mills+t)
  selected <- d==1;n1 <- sum(selected)
  w <- cbind(outcome_design,mills)[selected,,drop=FALSE]
  outcome <- lm.fit(w,y[selected]);b <- unname(outcome$coefficients)
  alpha <- tail(b,1);sse <- sum(outcome$residuals^2)
  sigma <- sqrt(sse/n1+alpha^2*mean(delta[selected]));rho <- alpha/sigma
  stopifnot(abs(rho)<1,sigma>0,outcome$rank==ncol(w))
  covariance <- reference$heckitVcov(w,selection_design[selected,,drop=FALSE],first$covariance,rho,delta[selected],sigma)
  invisible(chol(covariance))
  inference(case,'selection',first$theta,first$covariance)
  inference(case,'outcome',b,covariance)
  for(name in c('n','selected_count','sigma','rho','outcome_sse','selection_log_likelihood')) {
    value <- switch(name,n=length(d),selected_count=n1,sigma=sigma,rho=rho,outcome_sse=sse,selection_log_likelihood=first$ll)
    emit(case,name,value)
  }
  latent <- drop(outcome_design%*%head(b,-1))
  data_rows[[case]] <- data.frame(case=case,row=i,d=d,y=ifelse(selected,y,NA),x=x,z=z)
  predictions[[case]] <- data.frame(case=case,row=i,xb=latent,pr=p,mills=mills,
    ycond=latent+alpha*mills,yexpected=p*latent+alpha*dnorm(t))
}
write.csv(do.call(rbind,data_rows),file.path(args[1],'heckman_data.csv'),row.names=FALSE,na='')
write.csv(do.call(rbind,quantities),file.path(args[1],'heckman_reference.csv'),row.names=FALSE)
write.csv(do.call(rbind,predictions),file.path(args[1],'heckman_predictions.csv'),row.names=FALSE)
cat(R.version.string,'; unmodified sampleSelection1.2-16 heckitVcov; deterministic360-row controls\n')
