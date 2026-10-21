# Development-only base-R Poisson/NB2 PMF and CDF reference.
# Rscript crates/tabdat-stats/tests/fixtures/count_point_reference.R
options(digits=17)
root <- 'crates/tabdat-stats/tests/fixtures/'
rows <- list()
for(family in c('poisson','nb2')) for(alpha in if(family=='poisson') NA_real_ else c(exp(-11.9),.69,exp(11.9))) {
  for(mu in c(0,1e-300,1e-8,.306,3,23.5,750,1000,1e6)) for(k in c(0,1,9,99)) {
    pmf <- if(family=='poisson') dpois(k,mu) else dnbinom(k,size=1/alpha,mu=mu)
    cdf <- if(family=='poisson') ppois(k,mu) else pnbinom(k,size=1/alpha,mu=mu)
    rows[[length(rows)+1]] <- data.frame(family,alpha,mu,k,pmf,cdf)
  }
}
write.csv(do.call(rbind,rows),paste0(root,'count_point_reference.csv'),row.names=FALSE,quote=FALSE)
# Use the already independently audited, polished R offset-fit parameters. This
# isolates distributional prediction from estimator stopping-rule differences.
d <- read.csv('examples/nb2-offset/nb2_offset.csv')
rows <- list()
for(family in c('poisson','nb2')) {
  r <- read.csv(paste0(root,if(family=='poisson') 'poisson_offset_reference.csv' else 'nb2_offset_reference.csv'))
  r <- r[r$case=='multiple_model',]
  beta <- r$value[r$quantity=='coefficient_value']; stopifnot(length(beta)==3)
  alpha <- if(family=='poisson') NA_real_ else r$value[r$quantity=='alpha']
  # Current values change without refitting; reverse row order and offset shift.
  current <- d[36:1,]; current$x <- current$x+.2; current$log_time <- current$log_time-.3
  mu <- exp(drop(model.matrix(~x+z,current)%*%beta)+current$log_time)
  for(i in seq_len(nrow(current))) for(k in c(0,2,99)) {
    pmf <- if(family=='poisson') dpois(k,mu[i]) else dnbinom(k,size=1/alpha,mu=mu[i])
    cdf <- if(family=='poisson') ppois(k,mu[i]) else pnbinom(k,size=1/alpha,mu=mu[i])
    rows[[length(rows)+1]] <- data.frame(family,id=current$id[i],k,pmf,cdf)
  }
}
write.csv(do.call(rbind,rows),paste0(root,'count_point_workflow_reference.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,';',length(rows)*2,'workflow predictions\n')
