# Development-only R/plm reference; no source data/program/log redistributed.
# Rscript random_effects_cluster_reference.R MOM.dat fresh_reference_directory
options(digits=17)
library(plm)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L,!file.exists(args[2]))
v <- scan(args[1],quiet=TRUE)
stopifnot(length(v)==5320L*8L,all(is.finite(v)))
d <- as.data.frame(matrix(v,ncol=8,byrow=TRUE))
names(d) <- c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
d <- data.frame(source_row=seq_len(nrow(d)),d)
stopifnot(identical(d$id,rep(as.numeric(1:532),each=10)),
  identical(d$year,rep(as.numeric(1979:1988),532)),!anyDuplicated(d[c('id','year')]))
fit <- plm(lnhr~lnwg,data=d,index=c('id','year'),model='random',random.method='swar')
b <- unname(coef(fit)); V <- unname(vcovHC(fit,method="arellano",type="sss",cluster="group")); se <- sqrt(diag(V)); z <- b/se
W <- plm(lnhr~lnwg,data=d,index=c('id','year'),model='within')
B <- lm(lnhr~lnwg,data=aggregate(cbind(lnhr,lnwg)~id,d,mean))
ve <- sum(resid(W)^2)/(5320-532-1)
rawvu <- sum(resid(B)^2)/(532-2)-ve/10
vu <- max(rawvu,0); theta <- 1-sqrt(ve/(ve+10*vu))
stopifnot(max(abs(c(ve,vu)-fit$ercomp$sigma2))<1e-12,abs(theta-fit$ercomp$theta)<1e-12)
X <- cbind(1-theta,d$lnwg-theta*ave(d$lnwg,d$id))
ystar <- d$lnhr-theta*ave(d$lnhr,d$id)
explicit <- lm.fit(X,ystar)
bread <- solve(crossprod(X)); scores <- rowsum(X*as.numeric(explicit$residuals),d$id)
correction <- 532/531*5319/5318
stopifnot(max(abs(explicit$coefficients-b))<1e-10,
  max(abs(V-correction*bread%*%crossprod(scores)%*%bread))<1e-12)
rows <- list()
emit <- function(path,value) rows[[length(rows)+1L]] <<- data.frame(path,value=unname(value))
for(i in 1:2) {
  fields <- c(value=b[i],standard_error=se[i],statistic=z[i],p_value=2*pnorm(-abs(z[i])),
    ci_lower=b[i]-qnorm(.975)*se[i],ci_upper=b[i]+qnorm(.975)*se[i])
  for(name in names(fields)) emit(paste0('/coefficients/',i-1,'/',name),fields[[name]])
  for(j in 1:2) emit(paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
}
fields <- c(observation_count=5320,group_count=532,group_size=10,df_residual=5318,
  sigma_e_squared=ve,sigma_u_squared=vu,raw_sigma_u_squared=rawvu,theta=theta,rho=vu/(ve+vu),
  transformed_root_mse=sqrt(sum(explicit$residuals^2)/5318),transformed_rss=sum(explicit$residuals^2),
  wald_statistic=z[2]^2,wald_p_value=pchisq(z[2]^2,1,lower.tail=FALSE),wald_df=1)
for(name in names(fields)) emit(paste0('/',name),fields[[name]])
emit("/cluster/finite_sample_multiplier",correction)
dir.create(args[2])
write.csv(do.call(rbind,rows),file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
d$re_cluster_xb <- b[1]+b[2]*d$lnwg
d$re_cluster_residual <- d$lnhr-d$re_cluster_xb
d$re_robust_xb <- d$re_cluster_xb
d$re_robust_residual <- d$re_cluster_residual
write.csv(d,file.path(args[2],'predictions.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'; plm',as.character(packageVersion('plm')),';',length(rows),'scalars per alias; 5320 original/design/prediction rows\n')
