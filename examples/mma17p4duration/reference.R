# Independent development reference: R survival, not native/count implementation.
# Rscript reference.R duration.csv new_output_directory [original_ema1996.dta]
args <- commandArgs(TRUE)
stopifnot(length(args) %in% c(2,3),!file.exists(args[2]))
library(survival)
if(length(args)==3) {
  library(foreign)
  d<-read.dta(args[3],convert.factors=FALSE)
  d<-data.frame(source_row=seq_len(nrow(d)),d,check.names=FALSE)
} else d<-read.csv(args[1],check.names=FALSE)
raw<-d
x<-strsplit("reprate disrate ui rr_ui dr_ui logwage tenure slack abolpos explose stateur houshead married female child ychild nonwhite age schlt12 schgt12 smsa bluecoll mining constr transp trade fire services pubadmin year85 year87 year89 midatl encen wncen southatl escen wscen mountain pacific"," ")[[1]]
stopifnot(nrow(d)>41,all(is.finite(as.matrix(d))),all(d$spell>0),
  all(as.matrix(d[paste0("censor",1:4)]) %in% c(0,1)),
  all(rowSums(d[paste0("censor",1:4)])<=1),sum(d$censor1)>0,
  all(d$ui %in% c(0,1)),!anyDuplicated(d$source_row))
d$rr_ui<-d$reprate*d$ui;d$dr_ui<-d$disrate*d$ui
d$log_spell<-log(d$spell);d$original_ui<-d$ui
f<-as.formula(paste("Surv(spell,censor1)~",paste(x,collapse="+")))
s<-survreg(f,d,dist="exponential",robust=TRUE,
  control=survreg.control(maxiter=100,rel.tolerance=1e-12))
n<-nrow(d);beta<- -coef(s);names(beta)[1]<-"intercept"
v<-s$var*n/(n-1)
se<-sqrt(diag(v));z<-beta/se;q<-qnorm(.975)
null<-survreg(Surv(spell,censor1)~1,d,dist="exponential")
constant<-sum(d$censor1*log(d$spell))
wald<-drop(crossprod(beta[-1],solve(v[-1,-1],beta[-1])))
gates<-data.frame(path=character(),value=numeric())
add<-function(path,value) { gates[nrow(gates)+1,]<<-list(path,value) }
add("/observation_count",n);add("/count_sum",sum(d$censor1))
add("/log_likelihood",as.numeric(logLik(s))+constant)
add("/duration_density_log_likelihood",as.numeric(logLik(s)))
add("/null_log_likelihood",as.numeric(logLik(null))+constant)
add("/likelihood_ratio",2*(as.numeric(logLik(s))-as.numeric(logLik(null))))
add("/omnibus_wald/statistic",wald);add("/omnibus_wald/degrees_of_freedom",40)
add("/omnibus_wald/p_value",pchisq(wald,40,lower.tail=FALSE))
for(i in seq_along(beta)) {
  values<-c(value=beta[i],standard_error=se[i],statistic=z[i],
    p_value=2*pnorm(-abs(z[i])),confidence_lower=beta[i]-q*se[i],confidence_upper=beta[i]+q*se[i])
  names(values)<-c("value","standard_error","statistic","p_value","confidence_lower","confidence_upper")
  for(name in names(values)) add(paste0("/coefficients/",i-1,"/",name),values[name])
  for(j in seq_along(beta)) add(paste0("/covariance_matrix/",i-1,"/",j-1),v[i,j])
}
X<-model.matrix(f,d)
eta<-drop(X%*%beta);H<-d$spell*exp(eta)
# Separate package point prediction cross-check, including the AFT sign map.
stopifnot(max(abs(log(predict(s,type="response"))+eta))<1e-10)
ui0<-d;ui0$ui<-0;ui0$rr_ui<-0;ui0$dr_ui<-0
ui1<-d;ui1$ui<-1;ui1$rr_ui<-ui1$reprate;ui1$dr_ui<-ui1$disrate
H0<-d$spell*exp(drop(model.matrix(f,ui0)%*%beta))
H1<-d$spell*exp(drop(model.matrix(f,ui1)%*%beta))
pred<-d
pred$cumulative_hazard<-H;pred$log_cumulative_hazard<-eta+log(d$spell)
pred$ui0_cumulative_hazard<-H0;pred$ui1_cumulative_hazard<-H1
pred$hazard<-H/d$spell;pred$net_survival<-exp(-H);pred$mean_net_duration<-d$spell/H
pred$ui0_hazard<-H0/d$spell;pred$ui1_hazard<-H1/d$spell;pred$ui_hazard_ratio<-H1/H0
pred$net_survival_10<-exp(-10*H/d$spell);pred$ui0_net_survival_10<-exp(-10*H0/d$spell)
pred$ui1_net_survival_10<-exp(-10*H1/d$spell);pred$event_score_residual<-d$censor1-H
# Only publish the independent artifacts after all fitting/checks finish.
stopifnot(dir.create(args[2]))
write_numeric<-function(data,name) write.table(as.data.frame(lapply(data,function(x)
  if(is.numeric(x)) sprintf("%.17g",x) else x),check.names=FALSE),
  file.path(args[2],name),sep=",",quote=FALSE,row.names=FALSE)
write_numeric(gates,"reference.csv");write_numeric(pred,"predictions.csv")
write.table(as.data.frame(lapply(raw,function(x)sprintf("%a",x)),check.names=FALSE),
  file.path(args[2],"source_hex.csv"),sep=",",quote=FALSE,row.names=FALSE)
writeLines(names(beta),file.path(args[2],"parameter_names.txt"))
cat(R.version.string,"survival",as.character(packageVersion("survival")),
  "N",n,"K",length(beta),"gates",nrow(gates),"cells",nrow(pred)*ncol(pred),"\n")
