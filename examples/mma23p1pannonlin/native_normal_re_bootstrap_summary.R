# Development-only empirical bootstrap reference; independent base-R summaries.
a<-commandArgs(TRUE);stopifnot(length(a)==4,!file.exists(a[4]))
f<-read.csv(a[1],check.names=FALSE);o<-read.csv(a[2],check.names=FALSE)
v<-matrix(scan(a[3],what=character(),quiet=TRUE),ncol=25,byrow=TRUE)
X<-matrix(as.numeric(v[,11:15]),nrow=nrow(v));Y<-matrix(as.numeric(v[,21:25]),nrow=nrow(v))
B<-nrow(f);stopifnot(B>=2,B<=500,identical(f$replicate,seq_len(B)))
keys<-c('estimate_intercept','estimate_slope','estimate_lnvariance')
point<-o$value[match(keys,o$field)];values<-as.matrix(f[,keys]);stopifnot(!anyNA(point),all(is.finite(values)),all(is.finite(point)))
means<-colMeans(values);bias<-means-point;C<-cov(values);se<-sqrt(diag(C))
limits<-apply(values,2,quantile,probs=c(.025,.975),type=7,names=FALSE)
mu<-exp(point[1]+point[2]*X+exp(point[3])/2)
fields<-c(candidate_rows=length(Y),firms_per_draw=nrow(v),replications=B,
 estimate_intercept=point[1],estimate_slope=point[2],estimate_lnvariance=point[3],
 original_mu_mean=mean(mu),original_mu_min=min(mu),original_mu_max=max(mu),original_residual_mean=mean(Y-mu),
 mean_intercept=means[1],mean_slope=means[2],mean_lnvariance=means[3],
 bias_intercept=bias[1],bias_slope=bias[2],bias_lnvariance=bias[3],
 covariance_aa=C[1,1],covariance_ab=C[1,2],covariance_at=C[1,3],covariance_bb=C[2,2],covariance_bt=C[2,3],covariance_tt=C[3,3],
 se_intercept=se[1],se_slope=se[2],se_lnvariance=se[3],
 lower_intercept=limits[1,1],upper_intercept=limits[2,1],lower_slope=limits[1,2],upper_slope=limits[2,2],lower_lnvariance=limits[1,3],upper_lnvariance=limits[2,3])
stopifnot(length(fields)==31,all(is.finite(fields)))
names(fields)<-sub('\\.estimate_(intercept|slope|lnvariance)$','',names(fields))
writeLines(c('field,value',sprintf('%s,%.17g',names(fields),unname(fields))),a[4])
cat(R.version.string,';',B,'joint R refits; full B-1 covariance and type7 marginal percentile95\n')
