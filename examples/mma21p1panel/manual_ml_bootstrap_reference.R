# Development-only independent ML profile, person-index oracle and R QR bootstrap.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2,!file.exists(a[2]))
script<-sub('^--file=','',commandArgs()[grepl('^--file=',commandArgs())])
repo<-normalizePath(file.path(dirname(script),'../..'))
v<-scan(a[1],quiet=TRUE);stopifnot(length(v)>0,length(v)%%8==0,all(is.finite(v)))
d<-as.data.frame(matrix(v,ncol=8,byrow=TRUE));names(d)<-c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
G<-length(unique(d$id));N<-nrow(d);K<-2L;B<-500L
stopifnot(G>=3,G<=532,!anyDuplicated(d[c('id','year')]),all(table(d$id)==10),
  all(d$id==floor(d$id)),all(d$id>=1 & d$id<=532),all(d$year%in%1979:1988),
  all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
d$source_row<-seq_len(N);d$mean_lnhr<-ave(d$lnhr,d$id);d$mean_lnwg<-ave(d$lnwg,d$id)
d$within_lnhr<-d$lnhr-d$mean_lnhr;d$within_lnwg<-d$lnwg-d$mean_lnwg
profile<-function(eta,details=FALSE) {
  root<-exp(-eta/2);X<-cbind(root,d$within_lnwg+root*d$mean_lnwg)
  y<-d$within_lnhr+root*d$mean_lnhr;fit<-lm.fit(X,y);stopifnot(fit$rank==2)
  b<-unname(fit$coefficients);rss<-sum(fit$residuals^2);e<-rss/N
  stopifnot(is.finite(e),e>0)
  ll<--(N*(log(2*pi)+1+log(e))+G*eta)/2
  between<-d$mean_lnhr-b[1]-b[2]*d$mean_lnwg
  score<-(N*exp(-eta)*sum(between^2)/rss-G)/2
  if(details) return(c(intercept=b[1],slope=b[2],e=e,u=e*expm1(eta)/10,theta=1-root,ll=ll,score=score))
  c(ll=ll,score=score)
}
grid<-seq(0,24,by=.5);values<-vapply(grid,profile,c(ll=0,score=0))
crossings<-which(values['score',-length(grid)]>0 & values['score',-1]<=0)
roots<-vapply(crossings,function(i) uniroot(function(x) profile(x)['score'],c(grid[i],grid[i+1]),tol=1e-12)$root,0)
candidates<-c(0,roots);eta<-candidates[which.max(vapply(candidates,function(x) profile(x)['ll'],0))]
ans<-profile(eta,TRUE);stopifnot(values['score',length(grid)]<0,values['ll',length(grid)]<ans['ll'],
  if(eta==0) ans['score']<=1e-8 else abs(ans['score'])<1e-8)
S<-diag(ans['e'],10)+matrix(ans['u'],10,10);U<-chol(S)
ordered<-d[order(d$id,d$year),];r<-matrix(ordered$lnhr-ans['intercept']-ans['slope']*ordered$lnwg,nrow=10)
ll<--(N*log(2*pi)+G*2*sum(log(diag(U)))+sum(forwardsolve(t(U),r)^2))/2
stopifnot(abs(ll-ans['ll'])<1e-8)
d$population_xb<-ans['intercept']+ans['slope']*d$lnwg;d$population_residual<-d$lnhr-d$population_xb
d$sigma_e_squared<-ans['e'];d$sigma_u_squared<-ans['u'];d$theta<-ans['theta']
d$quasi_lnhr<-d$lnhr-ans['theta']*d$mean_lnhr;d$quasi_lnwg<-d$lnwg-ans['theta']*d$mean_lnwg
X<-cbind(1,d$quasi_lnwg);fit<-lm.fit(X,d$quasi_lnhr);stopifnot(fit$rank==K)
beta<-unname(fit$coefficients);stopifnot(max(abs(beta-c((1-ans['theta'])*ans['intercept'],ans['slope'])))<=1e-9)
# Native cluster order is first appearance in the original retained source sample.
members<-lapply(unique(d$id),function(id) which(d$id==id))
index_file<-tempfile(fileext='.csv')
status<-system2('python3',c(shQuote(file.path(repo,'crates/tabdat-stats/tests/fixtures/bootstrap/draws.py')),'--draws',10001,G,B,shQuote(index_file)))
stopifnot(status==0);indices<-read.csv(index_file,colClasses='character');unlink(index_file)
stopifnot(nrow(indices)==B)
bs<-t(vapply(indices$indices,function(text) {
  chosen<-as.integer(strsplit(text,':',fixed=TRUE)[[1]])+1
  stopifnot(length(chosen)==G,all(chosen>=1),all(chosen<=G))
  ix<-unlist(members[chosen],use.names=FALSE)
  point<-lm.fit(X[ix,,drop=FALSE],d$quasi_lnhr[ix]);stopifnot(point$rank==2)
  unname(point$coefficients)
},numeric(2)))
V<-cov(bs);means<-colMeans(bs);rows<-list()
emit<-function(model,path,value) rows[[length(rows)+1L]]<<-data.frame(model,path,value=sprintf('%.17g',unname(value)))
for(k in c('sigma_e_squared','sigma_u_squared','theta','log_likelihood')) {
  key<-switch(k,sigma_e_squared='e',sigma_u_squared='u',theta='theta',log_likelihood='ll')
  emit('canonical',paste0('/',k),ans[key])
}
for(i in 1:2) {
  ci<-quantile(bs[,i],c(.025,.975),type=7,names=FALSE)
  vals<-c(estimate=beta[i],bootstrap_mean=means[i],bias=means[i]-beta[i],standard_error=sqrt(V[i,i]),ci_lower=ci[1],ci_upper=ci[2])
  for(k in names(vals))emit('bootstrap',paste0('/coefficients/',i-1,'/',k),vals[k])
  for(j in 1:2)emit('bootstrap',paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
}
emit('bootstrap','/observation_count',N);emit('bootstrap','/replications',B)
d$quasi_fitted<-fit$fitted.values;d$quasi_residual<-fit$residuals
cols<-c('source_row','lnhr','lnwg','kids','ageh','agesq','disab','id','year','population_xb','population_residual','mean_lnhr','mean_lnwg','within_lnhr','within_lnwg','sigma_e_squared','sigma_u_squared','theta','quasi_lnhr','quasi_lnwg','quasi_fitted','quasi_residual')
dir.create(a[2]);write.table(do.call(rbind,rows),file.path(a[2],'reference.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(as.data.frame(lapply(d[cols],function(x)sprintf('%.17g',x))),file.path(a[2],'predictions.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(as.data.frame(lapply(as.data.frame(bs),function(x)sprintf('%.17g',x))),file.path(a[2],'refits.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.csv(indices,file.path(a[2],'draws.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'fields',length(rows),'N',N,'G',G,'B',B,'theta',ans['theta'],'bootstrap SE',sqrt(diag(V)),'\n')
