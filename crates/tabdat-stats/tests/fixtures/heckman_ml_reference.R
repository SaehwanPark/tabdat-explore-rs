# Independent Base R reference for the owned Heckman ML kernel tests.
# Run from the repository root:
# Rscript crates/tabdat-stats/tests/fixtures/heckman_ml_reference.R \
#   crates/tabdat-stats/tests/fixtures/heckman_data.csv /tmp/heckman_ml_reference.csv
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L)
data <- read.csv(args[1], na.strings="", check.names=FALSE)
quantities <- list()
add <- function(case, quantity, i, j, value) {
  quantities[[length(quantities)+1L]] <<- data.frame(
    case=case, quantity=quantity, i=i, j=j, value=value,
    stringsAsFactors=FALSE
  )
}

for (case in c("exclusion", "no_constant")) {
  rows <- data[data$case==case,]
  d <- rows$d
  constant <- case != "no_constant"
  x <- if (constant) cbind(1, rows$x) else as.matrix(rows["x"])
  z <- if (constant) cbind(1, rows$x, rows$z) else as.matrix(rows[c("x","z")])
  selected <- which(d==1)
  y <- rows$y[selected]
  xo <- x[selected,,drop=FALSE]
  n <- length(d); p <- ncol(xo); q <- ncol(z)
  probit <- glm.fit(x=z, y=d, family=binomial(link="probit"),
                    control=glm.control(epsilon=1e-13, maxit=100))
  gamma0 <- probit$coefficients
  zi0 <- drop(z %*% gamma0)
  mills0 <- dnorm(zi0[selected]) / pnorm(zi0[selected])
  delta0 <- mills0 * (mills0 + zi0[selected])
  ols0 <- lm.fit(x=cbind(xo,mills0), y=y)
  beta0 <- ols0$coefficients[seq_len(p)]
  alpha0 <- tail(ols0$coefficients,1L)
  sigma0 <- sqrt(mean(ols0$residuals^2) + alpha0^2*mean(delta0))
  rho0 <- alpha0/sigma0
  yscale <- sqrt(mean((y-mean(y))^2))
  xscale <- sqrt(colMeans(xo^2))
  zscale <- sqrt(colMeans(z^2))
  scales <- c(yscale/xscale, 1/zscale, 1, 1)
  if (constant) scales[1] <- yscale
  if (constant) scales[p+1L] <- 1
  initial <- c(beta0,gamma0,log(sigma0),atanh(rho0))

  value <- function(psi) {
    theta <- psi*scales
    beta <- theta[seq_len(p)]
    gamma <- theta[p+seq_len(q)]
    log_sigma <- theta[p+q+1L]
    atanh_rho <- theta[p+q+2L]
    if (!is.finite(log_sigma) || abs(atanh_rho)>8) return(1e100)
    sigma <- exp(log_sigma); rho <- tanh(atanh_rho)
    e <- (y-drop(xo%*%beta))/sigma
    zi <- drop(z%*%gamma)
    conditional <- (zi[selected]+rho*e)/sqrt(1-rho^2)
    ll <- sum(dnorm(e,log=TRUE)-log_sigma+pnorm(conditional,log.p=TRUE)) +
      sum(pnorm(-zi[d==0],log.p=TRUE))
    -ll
  }
  score_raw <- function(theta) {
    beta <- theta[seq_len(p)]
    gamma <- theta[p+seq_len(q)]
    log_sigma <- theta[p+q+1L]
    atanh_rho <- theta[p+q+2L]
    sigma <- exp(log_sigma); rho <- tanh(atanh_rho)
    c0 <- sqrt(1-rho^2)
    e <- (y-drop(xo%*%beta))/sigma
    zi <- drop(z%*%gamma)
    conditional <- (zi[selected]+rho*e)/c0
    mills <- dnorm(conditional)/pnorm(conditional)
    derivative_e <- -e + mills*rho/c0
    score_beta <- drop(crossprod(xo,-derivative_e/sigma))
    score_gamma <- drop(crossprod(z[d==0,,drop=FALSE],
                                  -dnorm(-zi[d==0])/pnorm(-zi[d==0]))) +
      drop(crossprod(z[selected,,drop=FALSE],mills/c0))
    score_lnsigma <- sum(-1-derivative_e*e)
    score_athrho <- sum(mills*(e+rho*zi[selected])/c0)
    c(score_beta,score_gamma,score_lnsigma,score_athrho)
  }
  gradient <- function(psi) -score_raw(psi*scales)*scales/n
  fit <- optim(initial/scales,value,gr=gradient,method="BFGS",
               control=list(maxit=500,reltol=1e-14))
  stopifnot(fit$convergence==0L)
  point <- fit$par
  information <- matrix(0,p+q+2L,p+q+2L)
  for (j in seq_along(point)) {
    step <- 2e-5*(1+abs(point[j]))
    plus <- minus <- point
    plus[j] <- plus[j]+step; minus[j] <- minus[j]-step
    plus_score <- score_raw(plus*scales)*scales
    minus_score <- score_raw(minus*scales)*scales
    information[,j] <- -(plus_score-minus_score)/(2*step)
  }
  information <- (information+t(information))/2
  covariance_scaled <- solve(information)
  covariance <- diag(scales)%*%covariance_scaled%*%diag(scales)
  theta <- point*scales
  se <- sqrt(diag(covariance))
  log_sigma <- theta[p+q+1L]; atanh_rho <- theta[p+q+2L]
  sigma <- exp(log_sigma); rho <- tanh(atanh_rho); lambda <- rho*sigma
  null_fit <- glm.fit(x=z,y=d,family=binomial(link="probit"),
                      control=glm.control(epsilon=1e-13,maxit=100))
  null_ols <- lm.fit(x=xo,y=y)
  null_sigma <- sqrt(mean(null_ols$residuals^2))
  null_zi <- drop(z%*%null_fit$coefficients)
  null_ll <- sum(d*pnorm(null_zi,log.p=TRUE)+(1-d)*pnorm(-null_zi,log.p=TRUE)) +
    sum(dnorm(null_ols$residuals/null_sigma,log=TRUE)-log(null_sigma))
  ll <- -fit$value
  lr <- 2*(ll-null_ll)

  for (i in seq_len(p)) {
    add(case,"outcome_value",i,0,theta[i])
    add(case,"outcome_standard_error",i,0,se[i])
  }
  for (i in seq_len(q)) {
    add(case,"selection_value",i,0,theta[p+i])
    add(case,"selection_standard_error",i,0,se[p+i])
  }
  for (i in seq_len(p+q+2L)) for (j in seq_len(p+q+2L))
    add(case,"covariance",i,j,covariance[i,j])
  add(case,"log_likelihood",0,0,ll)
  add(case,"independent_log_likelihood",0,0,null_ll)
  add(case,"independence_lr_chi_square",0,0,lr)
  add(case,"sigma",0,0,sigma)
  add(case,"rho",0,0,rho)
  add(case,"lambda",0,0,lambda)
  fitted_x <- if (constant) cbind(1,rows$x) else as.matrix(rows["x"])
  fitted_z <- if (constant) cbind(1,rows$x,rows$z) else as.matrix(rows[c("x","z")])
  for (row in c(1L,10L,100L,359L)) {
    xb <- drop(fitted_x[row,,drop=FALSE]%*%theta[seq_len(p)])
    index <- drop(fitted_z[row,,drop=FALSE]%*%theta[p+seq_len(q)])
    probability <- pnorm(index)
    imr <- dnorm(index)/probability
    predictions <- list(xb=xb,pr=probability,mills=imr,
                        ycond=xb+lambda*imr,
                        yexpected=probability*xb+lambda*dnorm(index))
    for (kind in names(predictions))
      add(case,paste0("prediction_",kind),row,0,predictions[[kind]])
  }
}
write.csv(do.call(rbind,quantities),args[2],row.names=FALSE,quote=TRUE)
