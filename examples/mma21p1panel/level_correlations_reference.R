# Development-only raw MOM independently reshaped observed-level correlations.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2,!file.exists(a[2]))
v<-scan(a[1],quiet=TRUE);stopifnot(length(v)>0,length(v)%%8==0,all(is.finite(v)))
d<-as.data.frame(matrix(v,ncol=8,byrow=TRUE));names(d)<-c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
N<-nrow(d);ids<-sort(unique(d$id));G<-length(ids);years<-1979:1988
stopifnot(G>=3,G<=532,!anyDuplicated(d[c('id','year')]),all(table(d$id)==10),
  all(d$id==floor(d$id)),all(d$id>=1 & d$id<=532),all(d$year%in%years),
  all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
d$source_row<-seq_len(N);output<-list()
for(name in c('lnhr','lnwg')) {
  M<-matrix(NA_real_,nrow=G,ncol=10)
  for(j in seq_along(years)) {
    z<-d[d$year==years[j],];stopifnot(nrow(z)==G,!anyDuplicated(z$id))
    M[,j]<-z[[name]][match(ids,z$id)]
  }
  stopifnot(all(is.finite(M)),all(apply(M,2,var)>0))
  C<-cor(M);out<-expand.grid(year_a=years,year_b=years);out<-out[order(out$year_a,out$year_b),]
  out$variable<-name;out$people<-G
  out$correlation<-mapply(function(x,y) C[x-1978,y-1978],out$year_a,out$year_b)
  output[[name]]<-out[c('variable','year_a','year_b','people','correlation')]
}
cols<-c('source_row','lnhr','lnwg','kids','ageh','agesq','disab','id','year')
dir.create(a[2]);write.table(as.data.frame(lapply(d[cols],function(x)sprintf('%.17g',x))),file.path(a[2],'rows.csv'),sep=',',row.names=FALSE,quote=FALSE)
out<-do.call(rbind,output);out[-1]<-lapply(out[-1],function(x)sprintf('%.17g',x))
write.table(out,file.path(a[2],'correlations.csv'),sep=',',row.names=FALSE,quote=FALSE)
cat(R.version.string,'N',N,'G',G,'correlations',nrow(out),'\n')
