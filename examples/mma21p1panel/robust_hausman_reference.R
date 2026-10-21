# Independent development-only R/plm/sandwich oracle for MMA p719 block7.
# Rscript robust_hausman_reference.R prepared.csv fresh_reference_directory [float32]
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)%in%c(2,3),!file.exists(args[2]))
d <- read.csv(args[1]); raw <- d
if(length(args)==3) {
  stopifnot(args[3]=='float32')
  for(n in c('lnhr','lnwg')) d[[n]] <- readBin(writeBin(d[[n]],raw(),size=4),double(),n=nrow(d),size=4)
}
stopifnot(!anyDuplicated(d[c('id','year')]),all(is.finite(d$id)),all(is.finite(d$year)))
d <- d[is.finite(d$lnhr)&is.finite(d$lnwg),]
sizes <- table(d$id); G <- length(sizes); N <- nrow(d); T <- unname(sizes[1])
stopifnot(G>3,T>=2,all(sizes==T))
re <- plm::plm(lnhr~lnwg,data=d,index=c('id','year'),model='random',random.method='swar')
theta <- as.numeric(re$ercomp$theta)
d$mean_lnhr <- ave(d$lnhr,d$id); d$mean_lnwg <- ave(d$lnwg,d$id)
d$within_lnhr <- d$lnhr-d$mean_lnhr; d$within_lnwg <- d$lnwg-d$mean_lnwg
d$theta <- theta; d$quasi_lnhr <- d$lnhr-theta*d$mean_lnhr; d$quasi_lnwg <- d$lnwg-theta*d$mean_lnwg
d <- d[order(d$source_row),]
fit <- lm(quasi_lnhr~quasi_lnwg+within_lnwg,d)
X <- model.matrix(fit); b <- unname(coef(fit)); e <- residuals(fit); K <- ncol(X)
B <- solve(crossprod(X)); scores <- rowsum(X*e,d$id)
V <- B%*%crossprod(scores)%*%B * G/(G-1)*(N-1)/(N-K)
stopifnot(max(abs(V-sandwich::vcovCL(fit,cluster=d$id,type='HC1',cadjust=TRUE)))<1e-10)
rows <- list(); emit <- function(result_type,path,value) rows[[length(rows)+1]] <<- data.frame(result_type,path,value=unname(value))
for(i in 1:K) {
  se <- sqrt(V[i,i]); t <- b[i]/se
  vals <- c(value=b[i],standard_error=se,statistic=t,p_value=2*pt(-abs(t),G-1))
  for(n in names(vals)) emit('RegressionResult',paste0('/coefficients/',i-1,'/',n),vals[[n]])
  for(j in 1:K) emit('RegressionResult',paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
}
vals <- c(observation_count=N,df_residual=N-K,r_squared=summary(fit)$r.squared,adjusted_r_squared=summary(fit)$adj.r.squared,root_mse=sqrt(sum(e^2)/(N-K)),log_likelihood=as.numeric(logLik(fit)))
for(n in names(vals)) emit('RegressionResult',paste0('/',n),vals[[n]])
se <- sqrt(V[3,3]); t <- b[3]/se
vals <- c(estimate=b[3],standard_error=se,statistic=t,p_value=2*pt(-abs(t),G-1),ci_lower=b[3]-qt(.975,G-1)*se,ci_upper=b[3]+qt(.975,G-1)*se,ci_level=95,df_residual=G-1)
for(n in names(vals)) emit('LincomResult',paste0('/',n),vals[[n]])
vals <- c(statistic=t*t,p_value=pf(t*t,1,G-1,lower.tail=FALSE),df=1,df_residual=G-1)
for(n in names(vals)) emit('TestResult',paste0('/',n),vals[[n]])
if(length(args)==3) {
  # Audit selected printed facts under float32 input storage; native float64
  # inference is compared separately, never rounded to make reference tests pass.
  facts <- c(b[1],sqrt(V[1,1]),b[2],sqrt(V[2,2]),b[3],sqrt(V[3,3]),t*t,theta)
  printed <- c(3.10763,.027293,.0668379,.0243001,.1008376,.0785137,1.6495074,.58470925)
  half_unit <- c(5e-6,5e-8,5e-8,5e-8,5e-8,5e-8,5e-8,5e-9)
  stopifnot(all(abs(facts-printed)<=half_unit))
  cat('PASS:8 selected author-printed coefficient/SE/H/theta facts under float32 ingestion\n')
}
d$hausman_fitted <- fitted(fit); d$hausman_residual <- e
dir.create(args[2]); write.csv(do.call(rbind,rows),file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(d,file.path(args[2],'predictions.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'plm',as.character(packageVersion('plm')),'sandwich',as.character(packageVersion('sandwich')),'N',N,'G',G,'theta',theta,'H',t*t,'F p',pf(t*t,1,G-1,lower.tail=FALSE),'chi-square p',pchisq(t*t,1,lower.tail=FALSE),'\n')
