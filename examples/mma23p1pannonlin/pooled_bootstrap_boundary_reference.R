# Independent base-R GLM controls for two bounded near-endpoint source designs.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==1L,!file.exists(args[1]))
rows <- lapply(c(9.9,-9.9),function(target) {
  high <- rep(c(FALSE,FALSE,FALSE,TRUE,TRUE),12)
  x <- ifelse(high,log(2)/abs(target),0)
  y <- if(target>0) ifelse(high,2,1) else ifelse(high,1,2)
  model <- glm(y~x,family=poisson(),control=glm.control(epsilon=1e-13,maxit=100))
  stopifnot(model$converged,max(abs(unname(coef(model))-c(if(target>0) 0 else log(2),target)))<1e-10)
  data.frame(target=target,intercept=unname(coef(model)[1]),slope=unname(coef(model)[2]))
})
write.csv(do.call(rbind,rows),args[1],row.names=FALSE,quote=FALSE)
