# Development-only dense multivariate/rescaling oracle, independent of native output.
options(digits=17)
args<-commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==3L,args[3]%in%c('multivariate','rescaled'),!file.exists(args[2]),requireNamespace('gee',quietly=TRUE))
tokens<-scan(args[1],what=character(),quiet=TRUE)
stopifnot(length(tokens)>0,length(tokens)%%25L==0)
src<-matrix(tokens,ncol=25,byrow=TRUE)
v<-suppressWarnings(matrix(as.numeric(src),ncol=25))
stopifnot(!anyNA(v[,-2]),all(is.finite(v[,-2])))
g<-nrow(v);T<-5L;n<-g*T;stopifnot(g>3,g<=346)
id<-rep(seq_len(g),each=T);period<-rep(seq_len(T),g)
x1<-as.vector(t(v[,11:15,drop=FALSE]));y<-as.vector(t(v[,21:25,drop=FALSE]))
x2<-cos(id*1.17+period*.71)+(period-3)/3
if(args[3]=='rescaled') x2<-x2*1e-6
stopifnot(all(y>=0),all(y<=1000),all(y==floor(y)))
d<-data.frame(id=id,period=period,y=y,x1=x1,x2=x2)
X<-model.matrix(y~x1+x2,d);p<-ncol(X);scales<-sqrt(colMeans(X^2));Xs<-sweep(X,2,scales,'/')
start<-coef(glm(y~x1+x2,data=d,family=poisson()))
fit<-gee::gee(y~x1+x2,id=id,data=d,family=poisson(),corstr='exchangeable',scale.fix=TRUE,scale.value=1,v4.4compat=TRUE,b=start,tol=1e-12,maxiter=100,silent=TRUE)
stopifnot(fit$error==0,all(is.finite(coef(fit))))
beta<-unname(coef(fit));eta<-drop(X%*%beta);mu<-exp(eta);residual<-y-mu;pearson<-residual/sqrt(mu)
S2<-sum(pearson^2);SG<-sum(tapply(pearson,id,sum)^2);rho<-(SG-S2)/((T-1)*S2)
stopifnot(abs(rho-fit$working.correlation[1,2])<1e-10,1-rho>1e-12,1+(T-1)*rho>1e-12)
R<-matrix(rho,T,T);diag(R)<-1
bread<-matrix(0,p,p);scores<-matrix(0,g,p);score_scale<-numeric(p)
for(i in seq_len(g)){
  index<-which(id==i);A<-diag(sqrt(mu[index]));V<-A%*%R%*%A
  D<-diag(mu[index])%*%Xs[index,,drop=FALSE]
  bread<-bread+t(D)%*%solve(V,D)
  scores[i,]<-drop(t(D)%*%solve(V,residual[index]))
  score_scale<-score_scale+colSums(abs(D))*max(abs(solve(V,residual[index])))
}
stopifnot(all(abs(colSums(scores))<=1e-10*(1+score_scale)))
working_scaled<-solve(bread)
working<-working_scaled/outer(scales,scales)
sandwich<-(g/(g-1))*working_scaled%*%crossprod(scores)%*%working_scaled/outer(scales,scales)
stopifnot(max(abs(working-fit$naive.variance)/(1+abs(working)))<1e-9,
  max(abs(sandwich-fit$robust.variance*g/(g-1))/(1+abs(sandwich)))<1e-9)
values<-c(observations=n,groups=g,group_size=T,rho=rho,pearson_normalizer=S2/n,working_scale=1,sandwich_correction=g/(g-1),zero_total_groups=sum(tapply(y,id,sum)==0))
parameter_names<-c('_cons','x1','x2')
for(method in c('working','sandwich')){
  covariance<-if(method=='working') working else sandwich
  se<-unname(sqrt(diag(covariance)));z<-beta/se
  for(j in seq_len(p)){
    item<-c(value=beta[j],standard_error=se[j],statistic=z[j],p_value=2*pnorm(-abs(z[j])),lower=beta[j]-qnorm(.975)*se[j],upper=beta[j]+qnorm(.975)*se[j])
    values<-c(values,setNames(item,paste(method,parameter_names[j],names(item),sep='.')))
    for(k in seq_len(p)) values<-c(values,setNames(covariance[j,k],paste(method,'covariance',j-1,k-1,sep='.')))
  }
  correlation<-covariance[-1,-1,drop=FALSE]/outer(se[-1],se[-1])
  wald<-drop(t(z[-1])%*%solve(correlation,z[-1]))
  values<-c(values,setNames(c(wald,pchisq(wald,p-1,lower.tail=FALSE)),paste(method,c('wald','wald_p_value'),sep='.')))
}
write_precise<-function(data,path){
  for(name in names(data)) if(is.numeric(data[[name]])) data[[name]]<-sprintf('%.17g',data[[name]])
  write.csv(data,path,row.names=FALSE,quote=FALSE,na='')
}
dir.create(args[2])
write_precise(d,file.path(args[2],'input.csv'))
write_precise(data.frame(field=names(values),value=unname(values)),file.path(args[2],'reference.csv'))
write_precise(data.frame(row=seq_len(n),id=id,period=period,mean=mu,xb=eta,residual=residual,pearson=pearson),file.path(args[2],'predictions.csv'))
raw_scores<-sweep(scores,2,scales,'*')
write_precise(data.frame(id=seq_len(g),score_cons=raw_scores[,1],score_x1=raw_scores[,2],score_x2=raw_scores[,3]),file.path(args[2],'scores.csv'))
version<-paste(R.version.string,'; gee',as.character(packageVersion('gee')),'; mode',args[3],'; dense block variance/score/inverse; fixed scale1/legacy Pearson; no native input')
writeLines(version,file.path(args[2],'version.txt'));cat(version,'\n')
