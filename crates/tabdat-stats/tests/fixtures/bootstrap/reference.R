# R lm/cov/type7-quantile reference using independently generated sample indices.
# Rscript crates/tabdat-stats/tests/fixtures/bootstrap/reference.R
options(digits=17)
root <- 'crates/tabdat-stats/tests/fixtures/bootstrap/'
d <- read.csv(paste0(root,'inputs.csv')); draws <- read.csv(paste0(root,'draws.csv'),colClasses='character')
rows <- list(); emit <- function(case,path,value) rows[[length(rows)+1]] <<- data.frame(case,path,value=unname(value))
for(case in c('multiple','noconstant','shifted')) {
  frame <- d; if(case=='shifted') frame$y <- frame$y+100
  formula <- if(case=='noconstant') y~x+z-1 else y~x+z
  fit <- lm(formula,frame); X <- model.matrix(fit); beta <- unname(coef(fit)); K <- length(beta)
  chosen <- draws[draws$case==case,]; B <- nrow(chosen)
  replicates <- t(vapply(chosen$indices,function(text) {
    index <- as.integer(strsplit(text,':',fixed=TRUE)[[1]])+1L
    r <- lm.fit(X[index,,drop=FALSE],frame$y[index]); stopifnot(r$rank==K)
    unname(r$coefficients)
  },numeric(K)))
  means <- colMeans(replicates); V <- cov(replicates)
  for(i in 1:K) {
    ci <- quantile(replicates[,i],c(.025,.975),type=7,names=FALSE)
    vals <- c(estimate=beta[i],bootstrap_mean=means[i],bias=means[i]-beta[i],standard_error=sqrt(V[i,i]),ci_lower=ci[1],ci_upper=ci[2])
    for(n in names(vals)) emit(case,paste0('/coefficients/',i-1,'/',n),vals[[n]])
    for(j in 1:K) emit(case,paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
  }
  emit(case,'/observation_count',nrow(frame)); emit(case,'/replications',B)
}
write.csv(do.call(rbind,rows),paste0(root,'reference.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,';',length(rows),'independent bootstrap quantities\n')
