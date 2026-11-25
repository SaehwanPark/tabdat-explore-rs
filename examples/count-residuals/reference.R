options(digits=17)
y <- c(0,0,2,0,0,3,0,3,1,3,8,3,0,0,0,2,1,2,0,0,2,1,1,2,0,5,4,1,1,0,0,2,2,0,1,0)
i <- 0:(length(y)-1L)
data <- data.frame(
  id=i+1L,
  y=y,
  x=-2+4*i/35,
  z=c(0,1,0.4)[(i %% 3)+1L],
  person=i %% 6L
)
data$log_time <- 0.2*data$x
data$exposure <- exp(data$log_time)
write.csv(data, "count_residuals.csv", row.names=FALSE, na="", quote=FALSE)

fit_control <- glm.control(epsilon=1e-14, maxit=100)
append_residuals <- function(model, fixed_term, fit, family) {
  current <- data
  current$x <- current$x+0.2
  current$log_time <- current$log_time-0.3
  current$exposure <- current$exposure*exp(-0.3)
  current$y <- current$y+1
  current$y[current$id==1L] <- NA_real_
  current$x[current$id==2L] <- NA_real_
  if (fixed_term == "offset") {
    current$log_time[current$id==3L] <- NA_real_
  } else {
    current$exposure[current$id==3L] <- NA_real_
  }

  mean <- predict(fit, newdata=current, type="response")
  response <- current$y-mean
  pearson <- (current$y-mean) / sqrt(if (model == "poisson") {
    mean
  } else {
    mean + (1/fit$theta)*mean^2
  })
  deviance <- rep(NA_real_, nrow(current))
  valid <- !is.na(current$y) & !is.na(mean) & mean >= 0
  zero <- valid & current$y == 0
  if (model == "poisson") {
    deviance[zero] <- -sqrt(2*mean[zero])
    positive <- valid & current$y > 0 & mean > 0
    unit <- 2*(current$y[positive]*log(current$y[positive]/mean[positive]) -
      (current$y[positive]-mean[positive]))
  } else {
    alpha <- 1/fit$theta
    deviance[zero] <- -sqrt(2/alpha*log1p(alpha*mean[zero]))
    positive <- valid & current$y > 0 & mean > 0
    unit <- 2*(current$y[positive]*log(current$y[positive]/mean[positive]) -
      (current$y[positive]+1/alpha)*
        log((1+alpha*current$y[positive])/(1+alpha*mean[positive])))
  }
  deviance[positive] <- sign(current$y[positive]-mean[positive]) * sqrt(pmax(0, unit))
  data.frame(
    model=model,
    fixed_term=fixed_term,
    id=current$id,
    y=current$y,
    mean=mean,
    response_residual=response,
    pearson_residual=pearson,
    deviance_residual=deviance
  )
}

references <- list()
for (model in c("poisson", "nbreg")) {
  for (fixed_term in c("offset", "exposure")) {
    if (fixed_term == "offset") {
      formula <- y ~ x + z + offset(log_time)
    } else {
      formula <- y ~ x + z + offset(log(exposure))
    }
    fit <- if (model == "poisson") {
      glm(formula, family=poisson(), data=data, control=fit_control)
    } else {
      MASS::glm.nb(formula, data=data, control=fit_control, maxit=100)
    }
    references[[length(references)+1L]] <- append_residuals(model, fixed_term, fit, model)
  }
}
write.csv(do.call(rbind, references), "count_residual_reference.csv",
          row.names=FALSE, na="NA", quote=FALSE)
