# R4.6.1/nnet7.3.20 independent full-joint four-class/two-predictor reference.
# Rscript multinomial_four_class_reference.R > multinomial_four_class_reference.csv
options(digits=17)
d <- data.frame(y=integer(),x=integer(),z=integer())
for(x in -1:1) for(z in -1:1) {
  counts <- c(1,2^(x+1)*3^(z+1),2^(1-x)*3^(z+1),2^(x+1)*3^(1-z))
  for(category in 1:4) d <- rbind(d,data.frame(y=rep(category,counts[category]),x=x,z=z))
}
d$y <- factor(d$y); n <- nrow(d); observed <- model.matrix(~y-1,d)
rows <- list()
for(intercept in c(TRUE,FALSE)) {
  mode <- if(intercept) "intercept" else "noconstant"
  formula <- if(intercept) y~x+z else y~x+z-1
  X <- model.matrix(formula,d); k <- ncol(X)
  fit <- nnet::multinom(formula,d,trace=FALSE,Hess=TRUE,maxit=2000,reltol=1e-14,abstol=1e-14)
  stopifnot(fit$convergence==0L)
  theta <- as.vector(t(coef(fit)))
  probabilities <- function(theta) {
    eta <- cbind(0,X%*%t(matrix(theta,nrow=3,byrow=TRUE)))
    weights <- exp(eta-apply(eta,1,max)); weights/rowSums(weights)
  }
  for(it in 1:20) {
    pr <- probabilities(theta);fit$fitted.values <- pr
    information <- nnet:::multinomHess(fit,Z=X)
    score <- as.vector(crossprod(X,observed[,-1]-pr[,-1]));step <- solve(information,score)
    if(max(abs(score))<1e-10 && max(abs(step))<1e-10) break
    theta <- theta+step
  }
  stopifnot(max(abs(score))<1e-10,max(abs(step))<1e-10)
  if(intercept) stopifnot(max(abs(theta-c(log(6),log(2),log(3),log(6),-log(2),log(3),log(6),log(2),-log(3))))<1e-10)
  V <- solve(information); ll <- sum(log(pr[cbind(seq_len(n),as.integer(d$y))]))
  null <- if(intercept) sum(log((tabulate(d$y)/n)[d$y])) else -n*log(4)
  lr <- 2*(ll-null);df <- 3*(k-as.integer(intercept))
  emit <- function(quantity,i,j,value) {
    rows[[length(rows)+1L]] <<- data.frame(mode=mode,quantity=quantity,i=i,j=j,value=value)
  }
  normal <- function(prefix,i,value,se) {
    emit(paste0(prefix,"value"),i,0,value);emit(paste0(prefix,"standard_error"),i,0,se)
    emit(paste0(prefix,"statistic"),i,0,value/se);emit(paste0(prefix,"p_value"),i,0,2*pnorm(-abs(value/se)))
    emit(paste0(prefix,"confidence_lower"),i,0,value-qnorm(.975)*se)
    emit(paste0(prefix,"confidence_upper"),i,0,value+qnorm(.975)*se)
  }
  emit("log_likelihood",0,0,ll);emit("null_log_likelihood",0,0,null)
  emit("likelihood_ratio",0,0,lr);emit("likelihood_ratio_p_value",0,0,pchisq(lr,df,lower.tail=FALSE))
  emit("mcfadden_r_squared",0,0,1-ll/null)
  for(i in seq_along(theta)) {
    normal("coefficient_",i-1,theta[i],sqrt(V[i,i]))
    for(j in seq_along(theta)) emit("covariance",i-1,j-1,V[i,j])
  }
  ame <- function(theta) {
    p <- probabilities(theta);beta <- rbind(0,matrix(theta,nrow=3,byrow=TRUE))
    effects <- matrix(0,4,2)
    for(r in 1:2) {
      slopes <- beta[,r+as.integer(intercept)]
      effects[,r] <- colMeans(p*(matrix(slopes,nrow=n,ncol=4,byrow=TRUE)-drop(p%*%slopes)))
    }
    as.vector(t(effects))
  }
  gradient <- matrix(0,8,length(theta))
  for(i in seq_along(theta)) {
    difference <- function(h) {
      plus <- minus <- theta;plus[i]<-plus[i]+h;minus[i]<-minus[i]-h
      (ame(plus)-ame(minus))/(2*h)
    }
    # Sixth-order Richardson permits a larger step: the high-z no-intercept
    # case amplifies cancellation from differences at h=1e-4. Keep the native
    # comparison tolerance fixed and reduce reference differentiation error.
    sixth <- function(h) {
      fourth <- (4*difference(h/2)-difference(h))/3
      fourth_half <- (4*difference(h/4)-difference(h/2))/3
      (16*fourth_half-fourth)/15
    }
    h <- .01*(1+abs(theta[i]))
    gradient[,i] <- sixth(h)
    stopifnot(max(abs(gradient[,i]-sixth(h/2)))<1e-11)
  }
  values <- ame(theta)
  for(i in 1:8) normal("effect_",i-1,values[i],sqrt(drop(gradient[i,]%*%V%*%gradient[i,])))
}
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE)
