#!/usr/bin/env Rscript
# Independent scalar Rubin rules on supplied, already-fitted estimates; no MI generation.
args<-commandArgs(trailingOnly=TRUE)
if(length(args)!=2) stop('usage: Rscript rubin_reference.R estimates.csv fresh_reference_directory')
if(file.exists(args[2])) stop('reference directory exists')
d<-read.csv(args[1],colClasses=c('integer','character','numeric','numeric'))
stopifnot(identical(names(d),c('imputation','parameter','estimate','standard_error')),
 all(!is.na(d$imputation)),all(d$imputation>=1 & d$imputation<=100),
 all(d$parameter %in% c('intercept','x1missing','x2missing')),
 all(is.finite(d$estimate)),all(abs(d$estimate)<=1e6),
 all(is.finite(d$standard_error)),all(d$standard_error>=1e-12 & d$standard_error<=1e6),
 !anyDuplicated(d[c('imputation','parameter')]))
m<-length(unique(d$imputation));stopifnot(m>=2,m<=100,
 identical(sort(unique(d$imputation)),seq_len(m)),nrow(d)==3*m,
 all(table(d$imputation)==3),all(table(d$parameter)==m))
out<-do.call(rbind,lapply(c('intercept','x1missing','x2missing'),function(parameter) {
 x<-d[d$parameter==parameter,];x<-x[order(x$imputation),];q<-x$estimate;u<-x$standard_error^2
 estimate<-mean(q);W<-mean(u);B<-var(q);T<-W+(1+1/m)*B;r<-(1+1/m)*B/W
 df<-if(B==0) NA_real_ else (m-1)*(1+1/r)^2
 se<-sqrt(T);statistic<-estimate/se
 stopifnot(is.finite(T),T>0,is.finite(r),is.finite(statistic),abs(statistic)<=1e6,
 B==0 || (is.finite(df) && df>=1 && df<=1e6))
 probability<-if(B==0) 2*pnorm(-abs(statistic)) else 2*pt(-abs(statistic),df)
 critical<-if(B==0) qnorm(.975) else qt(.975,df)
 fmi<-if(B==0) 0 else (r+2/(df+3))/(1+r)
 data.frame(parameter=parameter,imputations=m,estimate=estimate,within_variance=W,between_variance=B,total_variance=T,
 standard_error=se,relative_increase=r,degrees_freedom=df,statistic=statistic,p_value=probability,
 normal_or_t95_lower=estimate-critical*se,normal_or_t95_upper=estimate+critical*se,
 missing_information=fmi,relative_efficiency=1/(1+fmi/m),minimum=min(q),maximum=max(q),
 reference=if(B==0) 'normal_infinite_df_limit' else 'rubin_student_t_large_complete_df')
}))
dir.create(args[2]);options(digits=17)
write.csv(out,file.path(args[2],'rubin_marginal_report.csv'),row.names=FALSE,na='')
cat(R.version.string,'supplied imputations',m,'parameters',nrow(out),'\n')
