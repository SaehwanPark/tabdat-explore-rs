#!/usr/bin/env Rscript
# Independent base-R QR references on the actual ASCII values, including missingness.
args <- commandArgs(trailingOnly=TRUE)
if(length(args)!=2) stop('usage: Rscript reference.R input.asc fresh_reference_directory')
if(file.exists(args[2])) stop('reference directory exists')
d <- read.table(args[1],col.names=c('y','x1','x2'),na.strings='.',colClasses='numeric')
n <- nrow(d)
stopifnot(n>=4,n<=10000,all(is.finite(d$y)),all(abs(d$y)<=1e6),
 all(is.na(d$x1)|is.finite(d$x1)&abs(d$x1)<=1e6),all(is.na(d$x2)|is.finite(d$x2)&abs(d$x2)<=1e6),
 any(!is.na(d$x1)),any(!is.na(d$x2)))
m1<-mean(d$x1,na.rm=TRUE);m2<-mean(d$x2,na.rm=TRUE)
d$x1_filled<-ifelse(is.na(d$x1),m1,d$x1);d$x2_filled<-ifelse(is.na(d$x2),m2,d$x2)
cc<-lm(y~x1+x2,d,na.action=na.exclude);filled<-lm(y~x1_filled+x2_filled,d)
stopifnot(cc$rank==3,filled$rank==3,cc$df.residual>0,filled$df.residual>0,
 all(is.finite(c(coef(cc),coef(filled)))),summary(cc)$sigma>0,summary(filled)$sigma>0)
# Flat JSON-pointer records allow exact comparison of both actual CLI result types.
result<-list();add<-function(model,type,path,value) {
 result[[length(result)+1]]<<-data.frame(model=model,result=type,path=path,value=value)
}
for (model in c('complete','filled')) {
 fit<-if(model=='complete')cc else filled;s<-summary(fit);V<-vcov(fit);cf<-s$coefficients
 values<-c(observation_count=nobs(fit),df_residual=fit$df.residual,r_squared=s$r.squared,
 adjusted_r_squared=s$adj.r.squared,root_mse=s$sigma,log_likelihood=as.numeric(logLik(fit)))
 for(k in names(values)) add(model,'fit',paste0('/',k),unname(values[k]))
 for(i in 1:3) {
  for(j in 1:4) add(model,'fit',paste0('/coefficients/',i-1,'/',c('value','standard_error','statistic','p_value')[j]),cf[i,j])
  for(j in 1:3) add(model,'fit',paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
  interval<-cf[i,1]+c(-1,1)*qt(.975,fit$df.residual)*cf[i,2]
  for (pair in list(c('estimate',cf[i,1]),c('standard_error',cf[i,2]),c('statistic',cf[i,3]),c('p_value',cf[i,4]),c('ci_lower',interval[1]),c('ci_upper',interval[2]),c('df_residual',fit$df.residual)))
   add(model,paste0('lincom',i-1),paste0('/',pair[1]),as.numeric(pair[2]))
 }
}
cc_mean<-as.numeric(predict(cc,newdata=d));fill_mean<-as.numeric(predict(filled,newdata=d))
pred<-data.frame(source_row=seq_len(n),y=d$y,x1=d$x1,x2=d$x2,x1_missing=as.integer(is.na(d$x1)),x2_missing=as.integer(is.na(d$x2)),
 x1_filled=d$x1_filled,x2_filled=d$x2_filled,complete_mean=cc_mean,complete_residual=d$y-cc_mean,
 filled_mean=fill_mean,filled_residual=d$y-fill_mean)
diag<-data.frame(rows_total=n,observed_x1=sum(!is.na(d$x1)),observed_x2=sum(!is.na(d$x2)),missing_x1=sum(is.na(d$x1)),missing_x2=sum(is.na(d$x2)),
 complete_cases=sum(complete.cases(d[c('x1','x2')])),both_missing=sum(is.na(d$x1)&is.na(d$x2)),mean_x1_observed=m1,mean_x2_observed=m2)
dir.create(args[2]);options(digits=17)
write.csv(do.call(rbind,result),file.path(args[2],'fit.csv'),row.names=FALSE,na='')
write.csv(diag,file.path(args[2],'diagnostics.csv'),row.names=FALSE,na='')
write.csv(pred,file.path(args[2],'predictions.csv'),row.names=FALSE,na='')
cat('R',R.version.string,'rows',n,'complete',nobs(cc),'mean-filled',nobs(filled),'\n')
