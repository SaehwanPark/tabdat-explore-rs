# Development-only independent reference; no native/runtime dependency.
# Rscript weibull_reference.R duration.csv NEW_DIRECTORY [original_DTA_or_-] [noconstant]
args<-commandArgs(TRUE)
stopifnot(length(args)%in%2:4,!file.exists(args[2]))
no_intercept<-length(args)==4
if(no_intercept)stopifnot(args[4]=="noconstant")
library(survival)
if(length(args)>=3 && args[3]!="-") {
  library(foreign)
  raw<-read.dta(args[3],convert.factors=FALSE)
  raw<-data.frame(source_row=seq_len(nrow(raw)),raw,check.names=FALSE)
} else raw<-read.csv(args[1],check.names=FALSE)
d<-raw
x<-strsplit("reprate disrate ui rr_ui dr_ui logwage tenure slack abolpos explose stateur houshead married female child ychild nonwhite age schlt12 schgt12 smsa bluecoll mining constr transp trade fire services pubadmin year85 year87 year89 midatl encen wncen southatl escen wscen mountain pacific"," ")[[1]]
d$rr_ui<-d$reprate*d$ui;d$dr_ui<-d$disrate*d$ui
stopifnot(nrow(d)>42,all(is.finite(as.matrix(d))),all(d$spell>0),
  all(as.matrix(d[paste0("censor",1:4)])%in%c(0,1)),
  all(rowSums(d[paste0("censor",1:4)])<=1),sum(d$censor1)>0,
  all(d$ui%in%c(0,1)),!anyDuplicated(d$source_row))
f<-as.formula(paste("Surv(spell,censor1)~",paste(x,collapse="+"),if(no_intercept)"-1"else""))
s<-survreg(f,d,dist="weibull",robust=TRUE,
  control=survreg.control(maxiter=100,rel.tolerance=1e-12))
n<-nrow(d);k<-length(coef(s));a<-1/s$scale;beta<- -coef(s)*a
if(!no_intercept)names(beta)[1]<-"intercept"
q<-log(a);z95<-qnorm(.975);lnt<-log(d$spell);D<-d$censor1
X<-model.matrix(f,d);eta<-drop(X%*%beta);H<-exp(eta+a*lnt)
h<-exp(log(a)+eta+(a-1)*lnt);S<-exp(-H)
# The beta/lnsigma cross block is essential: beta=-b*exp(-lnsigma).
J<-matrix(0,k+1,k+1);diag(J)[seq_len(k)]<- -a
J[seq_len(k),k+1]<- -beta;J[k+1,k+1]<- -1
model_v<-J%*%s$naive.var%*%t(J)
robust_v<-J%*%s$var%*%t(J)*n/(n-1)
# A separate PH natural-shape score/Hessian route certifies the entire mapping.
Z<-cbind(X,lnt)
I<-crossprod(Z*sqrt(H));I[k+1,k+1]<-I[k+1,k+1]+sum(D)/a^2
scores<-cbind(X*(D-H),D/a+(D-H)*lnt)
B<-solve(I);T<-diag(k+1);T[k+1,k+1]<-1/a
independent_model<-T%*%B%*%T
independent_robust<-T%*%B%*%crossprod(scores)%*%B%*%T*n/(n-1)
stopifnot(max(abs(colSums(scores)))<1e-7,
  max(abs(independent_model-model_v)/(1+abs(model_v)))<1e-9,
  max(abs(independent_robust-robust_v)/(1+abs(robust_v)))<1e-9)
# Package prediction route does not use the PH cumulative-hazard expressions.
aft<-drop(predict(s,type="lp"))
package_survival<-1-psurvreg(d$spell,mean=aft,scale=s$scale,
  distribution="weibull")
stopifnot(max(abs(package_survival-S))<1e-12)
constant<-sum(D*lnt)
if(no_intercept) {
  # R survreg has no zero-location/shape-only model interface. Independently
  # solve the strictly decreasing NATURAL-shape null score by scalar bracketing,
  # not the native joint Newton algorithm or an arbitrary fixed shape.
  null_score<-function(a)sum(D)/a+sum(D*lnt)-sum(exp(a*lnt)*lnt)
  lo<-.Machine$double.eps;hi<-1
  stopifnot(null_score(lo)>0)
  while(null_score(hi)>0) { hi<-2*hi;stopifnot(is.finite(hi)) }
  null_shape<-uniroot(null_score,c(lo,hi),tol=1e-13,maxiter=1000)$root
  null_ll<-sum(D*(log(null_shape)+(null_shape-1)*lnt)-exp(null_shape*lnt))
} else {
  null<-survreg(Surv(spell,censor1)~1,d,dist="weibull",
    control=survreg.control(maxiter=100,rel.tolerance=1e-12))
  null_ll<-as.numeric(logLik(null))
}
slope<-if(no_intercept)seq_len(k)else 2:k
df<-length(slope)
gates_for<-function(V,robust) {
  gates<-data.frame(path=character(),value=numeric())
  add<-function(path,value) { gates[nrow(gates)+1,]<<-list(path,unname(value)) }
  se<-sqrt(diag(V)[seq_len(k)]);z<-beta/se;sq<-sqrt(V[k+1,k+1])
  add("/observation_count",n);add("/failure_count",sum(D));add("/censored_count",n-sum(D))
  add("/covariance_correction",if(robust)n/(n-1) else 1)
  add("/log_likelihood",as.numeric(logLik(s)))
  add("/normalized_log_likelihood",as.numeric(logLik(s))+constant)
  add("/null_log_likelihood",null_ll)
  add("/null_normalized_log_likelihood",null_ll+constant)
  lr<-2*(as.numeric(logLik(s))-null_ll)
  add("/likelihood_ratio",lr);add("/likelihood_ratio_df",df)
  if(!robust)add("/likelihood_ratio_p_value",pchisq(lr,df,lower.tail=FALSE))
  wald<-drop(crossprod(beta[slope],solve(V[slope,slope],beta[slope])))
  add("/omnibus_wald/statistic",wald);add("/omnibus_wald/degrees_of_freedom",df)
  add("/omnibus_wald/p_value",pchisq(wald,df,lower.tail=FALSE))
  for(i in seq_len(k)) {
    values<-c(value=beta[i],standard_error=se[i],statistic=z[i],
      p_value=2*pnorm(-abs(z[i])),confidence_lower=beta[i]-z95*se[i],confidence_upper=beta[i]+z95*se[i])
    names(values)<-c("value","standard_error","statistic","p_value","confidence_lower","confidence_upper")
    for(name in names(values))add(paste0("/coefficients/",i-1,"/",name),values[name])
  }
  shape<-c(value=a,log_value=q,standard_error=a*sq,log_standard_error=sq,
    log_statistic=q/sq,log_p_value=2*pnorm(-abs(q/sq)),
    confidence_lower=exp(q-z95*sq),confidence_upper=exp(q+z95*sq))
  for(name in names(shape))add(paste0("/shape/",name),shape[name])
  for(i in seq_len(k+1))for(j in seq_len(k+1))
    add(paste0("/covariance_matrix/",i-1,"/",j-1),V[i,j])
  gates
}
model_gates<-gates_for(model_v,FALSE);robust_gates<-gates_for(robust_v,TRUE)
point<-function(data,prefix) {
  e<-drop(model.matrix(f,data)%*%beta);H<-exp(e+a*log(data$spell))
  result<-data.frame(xb=e,hazard=exp(log(a)+e+(a-1)*log(data$spell)),
    cumulative_hazard=H,net_survival=exp(-H))
  names(result)<-paste0(prefix,names(result));result
}
ui0<-d;ui0$ui<-0;ui0$rr_ui<-0;ui0$dr_ui<-0
ui1<-d;ui1$ui<-1;ui1$rr_ui<-ui1$reprate;ui1$dr_ui<-ui1$disrate
pred<-cbind(raw,point(d,""),point(ui0,"ui0_"),point(ui1,"ui1_"))
stopifnot(dir.create(args[2]))
write_numeric<-function(data,name)write.table(as.data.frame(lapply(data,function(x)
  if(is.numeric(x))sprintf("%.17g",x)else x),check.names=FALSE),
  file.path(args[2],name),sep=",",quote=FALSE,row.names=FALSE)
write_numeric(model_gates,"model_based.csv");write_numeric(robust_gates,"robust.csv")
write_numeric(pred,"predictions.csv")
write.table(as.data.frame(lapply(raw,function(x)sprintf("%a",x)),check.names=FALSE),
  file.path(args[2],"source_hex.csv"),sep=",",quote=FALSE,row.names=FALSE)
writeLines(c(names(beta),"lnshape"),file.path(args[2],"parameter_names.txt"))
cat(R.version.string,"survival",as.character(packageVersion("survival")),
  "N",n,"P",k+1,"model gates",nrow(model_gates),"robust gates",nrow(robust_gates),
  "max score",max(abs(colSums(scores))),"covariance reconstruction",
  max(abs(independent_robust-robust_v)),"\n")
