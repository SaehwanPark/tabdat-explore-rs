# Development-only independent Cox/Breslow reference. Never a runtime dependency.
# Rscript cox_reference.R duration.csv NEW_DIRECTORY [original_DTA]
args<-commandArgs(TRUE);stopifnot(length(args)%in%2:3,!file.exists(args[2]))
library(survival)
if(length(args)==3){raw<-foreign::read.dta(args[3],convert.factors=FALSE);raw<-data.frame(source_row=seq_len(nrow(raw)),raw,check.names=FALSE)}else raw<-read.csv(args[1],check.names=FALSE)
d<-raw;d$rr_ui<-d$reprate*d$ui;d$dr_ui<-d$disrate*d$ui
x<-strsplit('reprate disrate ui rr_ui dr_ui logwage tenure slack abolpos explose stateur houshead married female child ychild nonwhite age schlt12 schgt12 smsa bluecoll mining constr transp trade fire services pubadmin year85 year87 year89 midatl encen wncen southatl escen wscen mountain pacific',' ')[[1]]
stopifnot(nrow(d)>40,all(is.finite(as.matrix(d))),all(d$spell>0),sum(d$censor1)>0,
 all(as.matrix(d[paste0('censor',1:4)])%in%c(0,1)),all(rowSums(d[paste0('censor',1:4)])<=1),all(d$ui%in%c(0,1)),!anyDuplicated(d$source_row))
f<-as.formula(paste('Surv(spell,censor1)~',paste(x,collapse='+')))
# Polish using only the independent R solution, never a native initial value.
control<-coxph.control(iter.max=100,eps=1e-13,toler.chol=1e-15)
s<-coxph(f,d,ties='breslow',robust=TRUE,x=TRUE,y=TRUE,model=TRUE,control=control)
s<-coxph(f,d,ties='breslow',robust=TRUE,x=TRUE,y=TRUE,model=TRUE,control=control,init=coef(s))
X<-s$x;stopifnot(identical(colnames(X),x),ncol(X)==40,all(is.finite(coef(s))))
n<-nrow(X);k<-ncol(X);b<-coef(s);reference<-colMeans(X);z95<-qnorm(.975)
Z<-sweep(X,2,reference);eta<-drop(Z%*%b);w<-exp(eta)
I<-matrix(0,k,k);score<-numeric(k);ui<-matrix(0,n,k);independent_ll<-sum(d$censor1*eta)
times<-sort(unique(d$spell[d$censor1==1]));baseline<-matrix(0,length(times),8)
colnames(baseline)<-c('time','risk_count','event_count','log_increment','increment','log_cumulative_hazard','cumulative_hazard','survival')
for(j in seq_along(times)){
 t<-times[j];risk<-which(d$spell>=t);ev<-which(d$spell==t & d$censor1==1);dj<-length(ev)
 S0<-sum(w[risk]);mu<-colSums(Z[risk,,drop=FALSE]*w[risk])/S0
 independent_ll<-independent_ll-dj*log(S0)
 I<-I+dj*(crossprod(Z[risk,,drop=FALSE]*sqrt(w[risk]))/S0-tcrossprod(mu))
 score<-score+colSums(Z[ev,,drop=FALSE])-dj*mu
 # FULL independent-spell score: failure-only or Z*(d-H) meat is incorrect.
 ui[ev,]<-ui[ev,,drop=FALSE]+sweep(Z[ev,,drop=FALSE],2,mu)
 ui[risk,]<-ui[risk,,drop=FALSE]-sweep(Z[risk,,drop=FALSE],2,mu)*(w[risk]*dj/S0)
 inc<-dj/S0;cum<-inc+if(j>1)baseline[j-1,'cumulative_hazard']else 0
 baseline[j,]<-c(t,length(risk),dj,log(inc),inc,log(cum),cum,exp(-cum))
}
B<-solve(I);V<-B%*%crossprod(ui)%*%B*n/(n-1)
# coxph(init=...) reports the supplied initial likelihood, NOT the zero-beta null.
null_ll<- -sum(baseline[,'event_count']*log(baseline[,'risk_count']))
stopifnot(abs(independent_ll-s$loglik[2])<1e-8,max(abs(score))<1e-7,max(abs(ui-residuals(s,type='score')))<1e-10,
 max(abs(B-s$naive.var))<1e-9,max(abs(V-s$var*n/(n-1)))<1e-9,
 max(abs(I%*%B-diag(k)))<1e-8)
# Basehaz uses R's dummy-zero reference; explicitly transform to ALL-X means.
bh<-basehaz(s,centered=TRUE);mapped<-bh$hazard[match(times,bh$time)]*exp(sum((reference-s$means)*b))
stopifnot(max(abs(mapped-baseline[,'cumulative_hazard']))<1e-9)
point<-function(data){
 lp<-drop(as.matrix(data[x])%*%b);H<-as.numeric(predict(s,newdata=data,type='expected'))
 index<-findInterval(data$spell,times);base<-c(0,baseline[,'cumulative_hazard'])[index+1]
 H2<-base*exp(drop(sweep(as.matrix(data[x]),2,reference)%*%b))
 stopifnot(max(abs(H-H2))<1e-9)
 data.frame(xb=lp,cumulative_hazard=H,net_survival=exp(-H),check.names=FALSE)
}
pred<-cbind(raw,point(d))
for(u in 0:1){v<-d;v$ui<-u;v$rr_ui<-v$reprate*v$ui;v$dr_ui<-v$disrate*v$ui;p<-point(v);names(p)<-paste0('ui',u,'_',names(p));pred<-cbind(pred,p)}
# Cox current-time expected counts do not depend on the supplied event indicator.
v<-d;v$censor1<-1-v$censor1;stopifnot(max(abs(point(v)$cumulative_hazard-pred$cumulative_hazard))<1e-12)
dir.create(args[2]);options(digits=17)
writeLines(x,file.path(args[2],'parameter_names.txt'))
write.csv(pred,file.path(args[2],'predictions.csv'),row.names=FALSE,quote=FALSE)
write.csv(as.data.frame(baseline),file.path(args[2],'baseline.csv'),row.names=FALSE,quote=FALSE)
write.csv(as.data.frame(ui),file.path(args[2],'score_residuals.csv'),row.names=FALSE,quote=FALSE)
hex<-as.data.frame(lapply(raw,function(v)sprintf('%a',v)),check.names=FALSE)
write.csv(hex,file.path(args[2],'source_hex.csv'),row.names=FALSE,quote=FALSE)
for(robust in c(FALSE,TRUE)){
 covariance<-if(robust)V else B;correction<-if(robust)n/(n-1)else 1
 se<-sqrt(diag(covariance));z<-b/se;lr<-2*(s$loglik[2]-null_ll)
 wald<-drop(t(b)%*%solve(covariance,b));flat<-list()
 put<-function(path,value){stopifnot(length(value)==1,is.finite(value));flat[[length(flat)+1]]<<-data.frame(path=path,value=value)}
 for(i in seq_len(k)){
  fields<-c(value=b[i],standard_error=se[i],statistic=z[i],p_value=2*pnorm(-abs(z[i])),confidence_lower=b[i]-z95*se[i],confidence_upper=b[i]+z95*se[i])
  # c() retains coefficient names; strip those suffixes for the public field keys.
  names(fields)<-c('value','standard_error','statistic','p_value','confidence_lower','confidence_upper')
  for(field in names(fields))put(paste0('/coefficients/',i-1,'/',field),fields[field])
  put(paste0('/baseline_reference_values/',i-1),reference[i])
  for(j in seq_len(k))put(paste0('/covariance_matrix/',i-1,'/',j-1),covariance[i,j])
 }
 for(i in seq_len(nrow(baseline)))for(field in colnames(baseline))put(paste0('/baseline/',i-1,'/',field),baseline[i,field])
 for(field in c('observation_count','failure_count','censored_count','distinct_exit_times','covariance_correction','log_likelihood','null_log_likelihood','likelihood_ratio','likelihood_ratio_df')){
  value<-switch(field,observation_count=n,failure_count=sum(d$censor1),censored_count=n-sum(d$censor1),distinct_exit_times=length(unique(d$spell)),covariance_correction=correction,log_likelihood=s$loglik[2],null_log_likelihood=null_ll,likelihood_ratio=lr,likelihood_ratio_df=k);put(paste0('/',field),value)
 }
 if(!robust)put('/likelihood_ratio_p_value',pchisq(lr,k,lower.tail=FALSE))
 put('/omnibus_wald/statistic',wald);put('/omnibus_wald/degrees_of_freedom',k);put('/omnibus_wald/p_value',pchisq(wald,k,lower.tail=FALSE))
 write.csv(do.call(rbind,flat),file.path(args[2],if(robust)'robust.csv'else'model_based.csv'),row.names=FALSE,quote=FALSE)
}
cat('Independent Cox/Breslow reference N',n,'K',k,'knots',length(times),'max score',max(abs(score)),'partial LL',format(s$loglik[2],digits=17),'\n')
