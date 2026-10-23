# R4.6.1/mlogit2.0.0/survival3.8.6 independent six-parameter exact-count fixture.
# R_LIBS_USER=/tmp/tabdat-nested-R-lib Rscript nested_choice_reference.R > nested_choice_reference.csv
library(mlogit)
library(survival)
stopifnot(packageVersion('mlogit')=='2.0.0')
options(digits=17)
cells <- list(list(p=c(0,1,0,0),q=c(0,0,0,1),count=c(6,12,16,48)),
  list(p=c(0,0,0,1),q=c(0,1,0,0),count=c(8,24,9,18)),
  list(p=c(1,0,0,1),q=c(0,0,1,0),count=c(12,6,75,50)))
wide <- list();long <- list();id<-0
for(cell in cells)for(income in 0:1)for(chosen in 1:4) {
  count <- cell$count[chosen]*if(chosen<=2)3^income else 1
  for(replication in seq_len(count)) {
    id<-id+1
    wide[[id]]<-data.frame(mode=chosen,y=income,
      p1=cell$p[1],p2=cell$p[2],p3=cell$p[3],p4=cell$p[4],
      q1=cell$q[1],q2=cell$q[2],q3=cell$q[3],q4=cell$q[4],
      px=cell$p[chosen],qx=cell$q[chosen],shore=as.numeric(chosen<=2))
    long[[id]]<-data.frame(id=id,alternative=1:4,chosen=as.numeric(chosen==1:4),
      p=cell$p,q=cell$q,dshore=c(1,1,0,0),dshorey=c(1,1,0,0)*income)
  }
}
w<-do.call(rbind,wide);d<-do.call(rbind,long)
stopifnot(id==704)
for(name in names(w))assign(name,w[[name]])
par<-c(log(2),log(3),log(2),log(3),2,3)
expr<-expression(bp*px+bq*qx+shore*(a+b*y)+
  shore*(rs-1)*log(exp(bp*p1+bq*q1)+exp(bp*p2+bq*q2))+
  (1-shore)*(rb-1)*log(exp(bp*p3+bq*q3)+exp(bp*p4+bq*q4))-
  log(exp(a+b*y)*(exp(bp*p1+bq*q1)+exp(bp*p2+bq*q2))^rs+
    (exp(bp*p3+bq*q3)+exp(bp*p4+bq*q4))^rb))
fn<-deriv3(expr,c('bp','bq','a','b','rs','rb'),function.arg=TRUE)
value<-do.call(fn,as.list(par))
H<-apply(attr(value,'hessian'),c(2,3),sum)
V<-solve(-H)
X<-lapply(1:4,function(j)as.matrix(d[d$alternative==j,c('p','q','dshore','dshorey')]))
Y<-lapply(1:4,function(j)as.logical(d$chosen[d$alternative==j]));names(X)<-names(Y)<-as.character(1:4)
rpar<-par;rpar[3:4]<-rpar[3:4]/par[5]
pkg<-getFromNamespace('lnl.nlogit','mlogit')(rpar,X,Y,weights=rep(1,704),gradient=TRUE,
  opposite=1,nests=list(shore=c('1','2'),boat=c('3','4')),unscaled=TRUE)
stopifnot(abs(as.numeric(pkg)-sum(value))<1e-9,max(abs(attr(pkg,'gradient')))<1e-9,
  max(abs(colSums(attr(value,'gradient'))))<1e-9)
restricted<-clogit(chosen~p+q+dshore+dshorey+strata(id),data=d,method='exact',
  control=coxph.control(eps=1e-12,toler.chol=1e-14,iter.max=100))
quantities<-list()
emit<-function(model,quantity,i,j,value) {
  quantities[[length(quantities)+1L]]<<-data.frame(model=model,quantity=quantity,i=i,j=j,value=value)
}
for(model in c('restricted','nested')) {
  beta<-if(model=='nested')par else coef(restricted)
  covariance<-if(model=='nested')V else vcov(restricted)
  emit(model,'log_likelihood',0,0,if(model=='nested')sum(value)else as.numeric(logLik(restricted)))
  for(i in seq_along(beta)) {
    se<-sqrt(covariance[i,i]);null<-if(i<=4)0 else 1;z<-(beta[i]-null)/se
    for(name in c('value','standard_error','statistic','p_value','confidence_lower','confidence_upper')) {
      emit(model,paste0('parameter_',name),i,0,switch(name,value=beta[i],standard_error=se,statistic=z,
        p_value=2*pnorm(-abs(z)),confidence_lower=beta[i]-qnorm(.975)*se,confidence_upper=beta[i]+qnorm(.975)*se))
    }
    for(j in seq_along(beta))emit(model,'covariance',i,j,covariance[i,j])
  }
}
lr<-2*(sum(value)-as.numeric(logLik(restricted)))
emit('nested','independence_likelihood_ratio',0,0,lr)
emit('nested','independence_p_value',0,0,pchisq(lr,2,lower.tail=FALSE))
write.csv(do.call(rbind,quantities),stdout(),row.names=FALSE,quote=FALSE)
