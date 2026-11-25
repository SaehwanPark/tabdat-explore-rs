# Independent current-row means and raw response residuals from base R/MASS.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==1L)

y <- c(0,0,2,0,0,3,0,3,1,3,8,3,0,0,0,2,1,2,0,0,2,1,1,2,0,5,4,1,1,0,0,2,2,0,1,0)
i <- 0:(length(y)-1L)
data <- data.frame(
  id=i+1L,
  y=y,
  x=-2+4*i/35,
  z=c(0,1,0.4)[(i %% 3)+1L]
)
data$log_time <- 0.2*data$x

fit_control <- glm.control(epsilon=1e-14,maxit=100)
poisson_fit <- glm(y ~ x + z + offset(log_time), family=poisson(), data=data,
  control=fit_control)
nb2_fit <- MASS::glm.nb(y ~ x + z + offset(log_time), data=data,
  control=fit_control, maxit=100)

current <- data
current$x <- current$x+0.2
current$log_time <- current$log_time-0.3
current$y <- current$y+1
current$y[current$id==1] <- NA_real_
current$x[current$id==2] <- NA_real_
current$log_time[current$id==3] <- NA_real_
rows <- data.frame(
  id=current$id,
  y=current$y,
  poisson_mean=predict(poisson_fit,newdata=current,type="response"),
  poisson_residual=current$y-predict(poisson_fit,newdata=current,type="response"),
  nb2_mean=predict(nb2_fit,newdata=current,type="response"),
  nb2_residual=current$y-predict(nb2_fit,newdata=current,type="response")
)
write.csv(rows,args[[1]],row.names=FALSE,quote=FALSE)
