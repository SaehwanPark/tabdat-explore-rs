args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=1L) stop('usage: reference.R DATA_OUTPUT.csv')
options(digits=17)

make_data <- function() {
  set.seed(506L)
  n <- 120L
  x <- seq(-1.75,1.75,length.out=n)
  treated <- as.integer(seq_len(n) %% 4L >= 2L)
  group <- rep(seq_len(20L),each=6L)
  probability <- plogis(-.25+.6*x+.45*treated)
  data.frame(y=rbinom(n,1L,probability),x=x,treated=treated,group=group)
}

data <- make_data()
write.csv(data,args[[1]],row.names=FALSE,quote=FALSE)

fit_one <- function(link,covariance) {
  X <- model.matrix(y~x+treated,data)
  y <- data$y
  fit <- glm(y~x+treated,data=data,family=binomial(link=link),
    control=glm.control(epsilon=1e-13,maxit=100L))
  beta <- unname(coef(fit))
  for (iteration in seq_len(30L)) {
    eta <- drop(X%*%beta)
    p <- if (link=='logit') plogis(eta) else pnorm(eta)
    q <- 1-p
    phi <- dnorm(eta)
    if (link=='logit') {
      score_slope <- y-p
      information_weight <- p*q
    } else {
      ratio <- phi/(p*q)
      score_slope <- (y-p)*ratio
      information_weight <- phi*ratio+(y-p)*ratio*(eta+(1-2*p)*ratio)
    }
    score <- drop(crossprod(X,score_slope))
    if (max(abs(score))<1e-13) break
    information <- crossprod(X,X*information_weight)
    beta <- beta+drop(solve(information,score))
  }
  eta <- drop(X%*%beta)
  p <- if (link=='logit') plogis(eta) else pnorm(eta)
  q <- 1-p
  phi <- dnorm(eta)
  if (link=='logit') {
    score_slope <- y-p
    information_weight <- p*q
  } else {
    ratio <- phi/(p*q)
    score_slope <- (y-p)*ratio
    information_weight <- phi*ratio+(y-p)*ratio*(eta+(1-2*p)*ratio)
  }
  information <- crossprod(X,X*information_weight)
  bread <- solve(information)
  scores <- X*score_slope
  covariance_matrix <- switch(covariance,
    model=bread,
    robust=length(y)/(length(y)-1)*bread%*%crossprod(scores)%*%bread,
    cluster={
      group_scores <- rowsum(scores,data$group,reorder=FALSE)
      group_count <- nrow(group_scores)
      group_count/(group_count-1)*bread%*%crossprod(group_scores)%*%bread
    })

  X0 <- X1 <- X
  X0[,'treated'] <- 0
  X1[,'treated'] <- 1
  eta0 <- drop(X0%*%beta)
  eta1 <- drop(X1%*%beta)
  p0 <- if (link=='logit') plogis(eta0) else pnorm(eta0)
  p1 <- if (link=='logit') plogis(eta1) else pnorm(eta1)
  derivative0 <- if (link=='logit') p0*(1-p0) else dnorm(eta0)
  derivative1 <- if (link=='logit') p1*(1-p1) else dnorm(eta1)
  gradient <- colMeans(X1*derivative1-X0*derivative0)
  effect <- mean(p1-p0)
  standard_error <- sqrt(drop(t(gradient)%*%covariance_matrix%*%gradient))
  statistic <- effect/standard_error
  data.frame(model=link,covariance=covariance,observation_count=length(y),
    group_count=if (covariance=='cluster') length(unique(data$group)) else NA_integer_,
    estimate=effect,standard_error=standard_error,statistic=statistic,
    p_value=2*pnorm(-abs(statistic)),confidence_lower=effect-1.959963984540054*standard_error,
    confidence_upper=effect+1.959963984540054*standard_error)
}

reference <- do.call(rbind,lapply(c('logit','probit'),function(link)
  do.call(rbind,lapply(c('model','robust','cluster'),function(covariance)
    fit_one(link,covariance)))))
write.csv(reference,stdout(),row.names=FALSE,quote=FALSE)
