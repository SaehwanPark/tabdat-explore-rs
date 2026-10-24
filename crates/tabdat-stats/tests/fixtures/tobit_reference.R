# Development-only synthetic Tobit references: R4.6.1 / survival3.8.6.
# Rscript crates/tabdat-stats/tests/fixtures/tobit_reference.R > .../tobit_reference.csv
suppressPackageStartupMessages(library(survival))
i <- 0:35
x1 <- (i%%6)-2.5;x2 <- (floor(i/6)-2.5)/3
noise <- c(-2.1,-1.2,-.7,-.1,.5,1.1,2.2,-1.6,.8,1.7,-.4)[((i*7+floor(i/6)*3)%%11)+1]
base <- pmax(0,.4+.8*x1-.7*x2+noise)
cases <- list(
  list(y=base,lower=0,intercept=TRUE),
  list(y=pmax(-.8,.8*x1-.7*x2+noise),lower=-.8,intercept=FALSE),
  list(y=8+.8*x1-.7*x2+noise,lower=-5,intercept=TRUE),
  list(y=base+5,lower=5,intercept=TRUE),
  list(y=base*1000,lower=0,intercept=TRUE))
# Below-limit recorded responses also contribute probability AT the limit.
cases[[2]]$y[which(cases[[2]]$y==-.8)[1]] <- -2
parameters <- c('b0','b1','b2','eta')
arguments <- c(parameters,'x1','x2','y','lower')
censored <- deriv3(~log(pnorm((lower-b0-b1*x1-b2*x2)/exp(eta))),parameters,function.arg=arguments)
uncensored <- deriv3(~-eta-.5*((y-b0-b1*x1-b2*x2)/exp(eta))^2-.5*log(2*pi),parameters,function.arg=arguments)
evaluate <- function(theta,d) {
  cpart <- censored(theta[1],theta[2],theta[3],theta[4],x1,x2,d$y,d$lower)
  upart <- uncensored(theta[1],theta[2],theta[3],theta[4],x1,x2,d$y,d$lower)
  event <- d$y>d$lower
  list(ll=sum(cpart[!event])+sum(upart[event]),
    score=colSums(attr(cpart,'gradient')[!event,,drop=FALSE])+colSums(attr(upart,'gradient')[event,,drop=FALSE]),
    information=-apply(attr(cpart,'hessian')[!event,,,drop=FALSE],c(2,3),sum)-apply(attr(upart,'hessian')[event,,,drop=FALSE],c(2,3),sum))
}
refine <- function(theta,free,d) {
  for(iteration in 1:20) {
    e <- evaluate(theta,d);information <- e$information[free,free,drop=FALSE];chol(information)
    step <- solve(information,e$score[free])
    if(max(abs(e$score[free])*pmax(1,abs(theta[free])))/36<1e-11 && max(abs(step)/pmax(1,abs(theta[free])))<1e-10)
      return(list(theta=theta,e=e,covariance=solve(information)))
    candidate <- theta;candidate[free] <- theta[free]+step
    stopifnot(evaluate(candidate,d)$ll>=e$ll-32*.Machine$double.eps*(1+abs(e$ll)))
    theta <- candidate
  }
  stop('independent observed-Newton refinement failed')
}
rows <- list()
emit <- function(model,quantity,value,i=0,j=0)rows[[length(rows)+1L]] <<- data.frame(model=model,quantity=quantity,i=i,j=j,value=as.numeric(value))
inference <- function(model,prefix,value,se,i) {
  z <- value/se
  values <- c(value=value,standard_error=se,statistic=z,p_value=2*pnorm(abs(z),lower.tail=FALSE),
    confidence_lower=value-qnorm(.975)*se,confidence_upper=value+qnorm(.975)*se)
  stopifnot(all(is.finite(values)),se>0,values['p_value']>0)
  for(name in names(values))emit(model,paste0(prefix,'_',name),values[name],i)
}
for(model in seq_along(cases)) {
  d <- cases[[model]];event <- d$y>d$lower
  data <- data.frame(observed=pmax(d$lower,d$y),event=event,x1=x1,x2=x2)
  formula <- if(d$intercept)Surv(observed,event,type='left')~x1+x2 else Surv(observed,event,type='left')~x1+x2-1
  f <- survreg(formula,data=data,dist='gaussian',control=survreg.control(maxiter=200,rel.tolerance=1e-12))
  free <- if(d$intercept)1:4 else 2:4
  initial <- if(d$intercept)c(unname(coef(f)),log(f$scale))else c(0,unname(coef(f)),log(f$scale))
  fit <- refine(initial,free,d);v <- fit$covariance;theta <- fit$theta
  stopifnot(abs(fit$e$ll-f$loglik[2])<1e-8,max(abs(v-vcov(f))/(1+abs(vcov(f))))<1e-7)
  if(d$intercept) {
    f0 <- survreg(Surv(observed,event,type='left')~1,data=data,dist='gaussian',control=survreg.control(maxiter=200,rel.tolerance=1e-12))
    null <- refine(c(unname(coef(f0)),0,0,log(f0$scale)),c(1,4),d)
    null_ll <- null$e$ll;stopifnot(abs(null_ll-f0$loglik[2])<1e-8)
  } else {
    # With location fixed at zero the likelihood is strictly concave in 1/sigma.
    # Bracket its score without imposing a fitted scale bound.
    score <- function(g)sum(event)/g-sum(d$y[event]^2)*g+sum(!event)*d$lower*exp(dnorm(d$lower*g,log=TRUE)-pnorm(d$lower*g,log.p=TRUE))
    low <- high <- 1/max(abs(d$y),abs(d$lower))
    while(score(low)<=0)low <- low/2
    while(score(high)>=0)high <- high*2
    g <- uniroot(score,c(low,high),tol=1e-14)$root
    stopifnot(abs(score(g))/36<1e-11)
    null_ll <- evaluate(c(0,0,0,-log(g)),d)$ll
  }
  beta <- theta[free[free!=4]];sigma <- exp(theta[4]);k <- length(beta);p <- k+1
  design <- if(d$intercept)cbind(1,x1,x2)else cbind(x1,x2)
  latent <- drop(design%*%beta);z <- (latent-d$lower)/sigma;probability <- pnorm(z);density <- dnorm(z)
  observed_mean <- d$lower+(latent-d$lower)*probability+sigma*density
  mean_gradient <- c(colMeans(design*(density/sigma)),mean(-z*density))
  for(j in 1:k)inference(model,'coefficient',beta[j],sqrt(v[j,j]),j)
  for(a in 1:p)for(b in 1:p)emit(model,'covariance',v[a,b],a,b)
  for(a in 1:p)emit(model,'mean_probability_gradient',mean_gradient[a],a)
  for(j in 1:2) {
    column <- j+as.integer(d$intercept)
    gradient <- beta[column]*mean_gradient;gradient[column] <- gradient[column]+mean(probability)
    variance <- drop(t(gradient)%*%v%*%gradient)
    inference(model,'ame',beta[column]*mean(probability),sqrt(variance),j)
  }
  lr <- 2*(fit$e$ll-null_ll)
  summary <- c(n=36,lower=d$lower,intercept=as.integer(d$intercept),censored=sum(!event),uncensored=sum(event),
    log_likelihood=fit$e$ll,null_log_likelihood=null_ll,likelihood_ratio=lr,likelihood_ratio_p_value=pchisq(lr,2,lower.tail=FALSE),
    sigma=sigma,log_sigma=theta[4],sigma_standard_error=sigma*sqrt(v[p,p]),log_sigma_standard_error=sqrt(v[p,p]),
    sigma_confidence_lower=exp(theta[4]-qnorm(.975)*sqrt(v[p,p])),sigma_confidence_upper=exp(theta[4]+qnorm(.975)*sqrt(v[p,p])),mean_probability=mean(probability))
  for(name in names(summary))emit(model,name,summary[name])
  for(row in 1:36) {
    emit(model,'input_y',d$y[row],row)
    emit(model,'input_x',x1[row],row,1);emit(model,'input_x',x2[row],row,2)
    emit(model,'latent',latent[row],row);emit(model,'probability',probability[row],row);emit(model,'mean',observed_mean[row],row)
  }
}
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE)
