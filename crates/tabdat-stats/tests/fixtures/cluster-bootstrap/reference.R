# Independent R lm/cov/type7 reference, consuming independent whole-group draws.
options(digits=17)
root <- 'crates/tabdat-stats/tests/fixtures/cluster-bootstrap/'
d <- read.csv(paste0(root,'inputs.csv')); draws <- read.csv(paste0(root,'draws.csv'),colClasses='character')
rows <- list();emit <- function(case,path,value) rows[[length(rows)+1]] <<- data.frame(case,path,value=unname(value))
for(case in c('balanced','unbalanced','noconstant','shifted')) {
 f <- d;if(case=='shifted') f$y <- f$y+100
 keys <- if(case=='balanced') f$balanced else f$unbalanced
 groups <- lapply(unique(keys),function(key) which(keys==key))
 fit <- lm(if(case=='noconstant') y~x+z-1 else y~x+z,f)
 X <- model.matrix(fit); beta <- unname(coef(fit)); K <- length(beta)
 chosen <- draws[draws$case==case,]; B <- nrow(chosen); sizes <- numeric(B)
 replicates <- t(vapply(seq_len(B),function(r) {
  group <- as.integer(strsplit(chosen$groups[r],':',fixed=TRUE)[[1]])+1L
  index <- unlist(groups[group],use.names=FALSE);sizes[r] <<- length(index)
  fit <- lm.fit(X[index,,drop=FALSE],f$y[index]);stopifnot(fit$rank==K)
  unname(fit$coefficients)
 },numeric(K)))
 V <- cov(replicates);means <- colMeans(replicates)
 for(i in seq_len(K)) {
  ci <- quantile(replicates[,i],c(.025,.975),type=7,names=FALSE)
  values <- c(estimate=beta[i],bootstrap_mean=means[i],bias=means[i]-beta[i],standard_error=sqrt(V[i,i]),ci_lower=ci[1],ci_upper=ci[2])
  for(n in names(values)) emit(case,paste0('/coefficients/',i-1,'/',n),values[[n]])
  for(j in seq_len(K)) emit(case,paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
 }
 emit(case,'/observation_count',nrow(f));emit(case,'/replications',B);emit(case,'/cluster_count',length(groups))
 for(r in seq_len(B)) emit(case,paste0('/replicate_observation_counts/',r-1),sizes[r])
 for(g in seq_along(groups)) emit(case,paste0('/cluster_sizes/',g-1),length(groups[[g]]))
}
write.csv(do.call(rbind,rows),paste0(root,'reference.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,';',length(rows),'cluster-bootstrap quantities\n')
