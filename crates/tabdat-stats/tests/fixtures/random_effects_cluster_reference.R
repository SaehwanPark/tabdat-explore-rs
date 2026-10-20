# Development-only oracle: R/plm balanced Swamy-Arora with explicit matrix audit.
# Rscript random_effects_cluster_reference.R output.csv
options(digits=17)
library(plm)
args <- commandArgs(trailingOnly=TRUE); stopifnot(length(args)==1L)
rows <- list()
emit <- function(case,path,value) rows[[length(rows)+1L]] <<- data.frame(case,path,value=unname(value))
for(case in c('balanced','interleaved','shifted','cluster_zero_component','singular_cluster')) {
  d <- expand.grid(time=0:3,id=1:6)
  d$x <- .3*d$id+.2*d$time+.07*((d$id*d$time+2)%%5)
  d$z <- .13*d$time^2+.03*((d$id+2*d$time)%%4)
  d$y <- 1+.4*d$id+.7*d$x-.3*d$z+.11*((d$id*d$time+3)%%7)
  if(case=='cluster_zero_component') d$y <- 1+.7*d$x-.3*d$z+.5*(-1)^d$time*(1+.02*d$id)
  if(case=='cluster_zero_component') d$y <- d$y+.01*((d$id+1)%%3)
  if(case=='singular_cluster') {
    group <- ifelse(d$id<=2,1,d$id)
    d$x <- d$time+.2*group; d$z <- d$time^2+.05*group^2
    sign <- ifelse(d$id==1,1,ifelse(d$id==2,-1,0))
    d$y <- 1+.7*d$x-.3*d$z+sign*(.01+d$time-1.5)
  }
  if(case=='shifted') d$y <- d$y+100
  if(case=='interleaved') d <- d[order(-d$time,-d$id),]
  fit <- plm(y~x+z,data=d,index=c('id','time'),model='random',random.method='swar')
  b <- unname(coef(fit)); V <- unname(vcovHC(fit,method="arellano",type="sss",cluster="group")); n <- nrow(d); G <- 6; p <- 2; T <- 4
  means <- aggregate(cbind(y,x,z)~id,data=d,mean)
  W <- lm(I(y-ave(y,id))~0+I(x-ave(x,id))+I(z-ave(z,id)),data=d)
  B <- lm(y~x+z,data=means)
  ve <- sum(resid(W)^2)/(n-G-p)
  rawvu <- sum(resid(B)^2)/(G-p-1)-ve/T
  vu <- max(rawvu,0); theta <- 1-sqrt(ve/(ve+T*vu))
  stopifnot(max(abs(c(ve,vu)-fit$ercomp$sigma2))<1e-10,abs(theta-fit$ercomp$theta)<1e-10)
  X <- cbind(1-theta,d$x-theta*ave(d$x,d$id),d$z-theta*ave(d$z,d$id))
  ystar <- d$y-theta*ave(d$y,d$id)
  direct <- lm.fit(X,ystar)
  bread <- solve(crossprod(X)); scores <- rowsum(X*as.numeric(direct$residuals),d$id)
  manual <- G/(G-1)*(n-1)/(n-p-1)*bread%*%crossprod(scores)%*%bread
  stopifnot(max(abs(direct$coefficients-b))<1e-10,max(abs(V-manual))<1e-10)
  for(i in 1:3) {
    se <- sqrt(V[i,i]); z <- b[i]/se
    emit(case,paste0('/gls/coefficients/',i-1,'/value'),b[i])
    emit(case,paste0('/gls/coefficients/',i-1,'/standard_error'),se)
    emit(case,paste0('/gls/coefficients/',i-1,'/statistic'),z)
    emit(case,paste0('/gls/coefficients/',i-1,'/p_value'),2*pnorm(-abs(z)))
    emit(case,paste0('/confidence_intervals/',i-1,'/0'),b[i]-qnorm(.975)*se)
    emit(case,paste0('/confidence_intervals/',i-1,'/1'),b[i]+qnorm(.975)*se)
    for(j in 1:3) emit(case,paste0('/gls/covariance/matrix/',i-1,'/',j-1),V[i,j])
  }
  for(i in seq_len(n)) {
    emit(case,paste0('/gls/fitted_values/',i-1),sum(X[i,]*b))
    emit(case,paste0('/gls/residuals/',i-1),direct$residuals[i])
  }
  if(case=='singular_cluster') {
    stopifnot(qr(cov2cor(V[-1,-1]))$rank==1L,rawvu<0)
    wald <- NA_real_
  } else wald <- drop(t(b[-1])%*%solve(V[-1,-1,drop=FALSE],b[-1]))
  values <- c(group_count=G,group_size=T,sigma_e_squared=ve,sigma_u_squared=vu,
    raw_sigma_u_squared=rawvu,theta=theta,rho=vu/(ve+vu),wald_statistic=wald,wald_p_value=pchisq(wald,p,lower.tail=FALSE))
  for(name in names(values)) if(is.finite(values[[name]])) emit(case,paste0('/',name),values[[name]])
  for(name in c('observation_count','degrees_of_freedom','root_mse','residual_sum_of_squares')) {
    value <- switch(name,observation_count=n,degrees_of_freedom=n-p-1,root_mse=sqrt(sum(direct$residuals^2)/(n-p-1)),residual_sum_of_squares=sum(direct$residuals^2))
    emit(case,paste0('/gls/fit_statistics/',name),value)
  }
}
write.csv(do.call(rbind,rows),args[1],row.names=FALSE,quote=FALSE)
cat(R.version.string,'; plm',as.character(packageVersion('plm')),';',length(rows),'scalar comparisons\n')
