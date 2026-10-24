# Independent R4.6.1/mlogit2.0.0/survival3.8.6 exact five-parameter fixture.
# 900 decisions; all six cell probabilities equal empirical positive frequencies.
library(mlogit)
library(survival)
stopifnot(packageVersion('mlogit')=='2.0.0')
options(digits=17)
cells<-list(list(p=c(0,2,1,3),q=c(1,0,1,0),k=2),
  list(p=c(0,2,0,2),q=c(1,0,2,1),k=3),list(p=c(0,2,0,2),q=c(1,0,1,0),k=1))
wide<-list();long<-list();id<-0
for(cell in cells)for(income in 0:1)for(chosen in 1:4) {
  counts<-c(18*3^income,32*3^income,9*cell$k,16*cell$k)
  for(replication in seq_len(counts[chosen])) {
    id<-id+1
    wide[[id]]<-data.frame(mode=chosen,y=income,k=cell$k,
      p1=cell$p[1],p2=cell$p[2],p3=cell$p[3],p4=cell$p[4],
      q1=cell$q[1],q2=cell$q[2],q3=cell$q[3],q4=cell$q[4],
      px=cell$p[chosen],qx=cell$q[chosen],shore=as.numeric(chosen<=2))
    long[[id]]<-data.frame(id=id,alternative=1:4,chosen=as.numeric(chosen==1:4),
      p=cell$p,q=cell$q,dshore=c(1,1,0,0),dshorey=c(1,1,0,0)*income)
  }
}
w<-do.call(rbind,wide);d<-do.call(rbind,long)
stopifnot(id==900,identical(as.numeric(table(w$mode)),c(216,384,108,192)))
for(name in names(w))assign(name,w[[name]])
par<-c(log(2),log(3),log(2),log(3),.5)
expr<-expression((bp*px+bq*qx)/r+shore*(a+b*y)+
  (r-1)*(shore*log(exp((bp*p1+bq*q1)/r)+exp((bp*p2+bq*q2)/r))+
    (1-shore)*log(exp((bp*p3+bq*q3)/r)+exp((bp*p4+bq*q4)/r)))-
  log(exp(a+b*y)*(exp((bp*p1+bq*q1)/r)+exp((bp*p2+bq*q2)/r))^r+
    (exp((bp*p3+bq*q3)/r)+exp((bp*p4+bq*q4)/r))^r))
fn<-deriv3(expr,c('bp','bq','a','b','r'),function.arg=TRUE)
value<-do.call(fn,as.list(par));H<-apply(attr(value,'hessian'),c(2,3),sum)
stopifnot(max(abs(colSums(attr(value,'gradient'))))<1e-9,min(eigen(-H,symmetric=TRUE,only.values=TRUE)$values)>0)
V<-solve(-H)
X<-lapply(1:4,function(j)as.matrix(d[d$alternative==j,c('p','q','dshore','dshorey')]))
Y<-lapply(1:4,function(j)as.logical(d$chosen[d$alternative==j]));names(X)<-names(Y)<-as.character(1:4)
pkg<-getFromNamespace('lnl.nlogit','mlogit')(par,X,Y,weights=rep(1,900),gradient=TRUE,
  opposite=1,nests=list(shore=c('1','2'),boat=c('3','4')),unscaled=FALSE,un.nest.el=TRUE)
expected<-cbind(18*3^w$y,32*3^w$y,9*w$k,16*w$k)/(25*(2*3^w$y+w$k))
stopifnot(abs(as.numeric(pkg)-sum(value))<1e-9,max(abs(attr(pkg,'gradient')))<1e-9,
  max(abs(attr(pkg,'probabilities')-expected))<1e-12)
restricted<-clogit(chosen~p+q+dshore+dshorey+strata(id),data=d,method='exact',
  control=coxph.control(eps=1e-12,toler.chol=1e-14,iter.max=100))
quantities<-list()
emit<-function(model,quantity,i,j,value) {
  quantities[[length(quantities)+1L]]<<-data.frame(model=model,quantity=quantity,i=i,j=j,value=value)
}
for(model in c('restricted','normalized_common')) {
  beta<-if(model=='normalized_common')par else coef(restricted)
  covariance<-if(model=='normalized_common')V else vcov(restricted)
  emit(model,'log_likelihood',0,0,if(model=='normalized_common')sum(value)else as.numeric(logLik(restricted)))
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
emit('normalized_common','independence_likelihood_ratio',0,0,lr)
emit('normalized_common','independence_p_value',0,0,pchisq(lr,1,lower.tail=FALSE))
write.csv(do.call(rbind,quantities),stdout(),row.names=FALSE,quote=FALSE)
