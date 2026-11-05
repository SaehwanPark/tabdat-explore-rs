# Development-only raw MOM, independent adjacent pairs and per-person R cor.
options(digits=17)
a<-commandArgs(trailingOnly=TRUE);stopifnot(length(a)==2,!file.exists(a[2]))
v<-scan(a[1],quiet=TRUE);stopifnot(length(v)>0,length(v)%%8==0,all(is.finite(v)))
d<-as.data.frame(matrix(v,ncol=8,byrow=TRUE));names(d)<-c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
ids<-sort(unique(d$id));G<-length(ids);stopifnot(G>=3,G<=532,!anyDuplicated(d[c('id','year')]),all(table(d$id)==10),
  all(d$id==floor(d$id)),all(d$id>=1 & d$id<=532),all(d$year%in%1979:1988),all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
d$source_row<-seq_len(nrow(d));d<-d[order(d$id,d$year),];p<-d[d$year>1979,];z<-d[d$year<1988,]
stopifnot(all(p$id==z$id),all(p$year==z$year+1),nrow(p)==9*G)
p$previous_source_row<-z$source_row;p$previous_year<-z$year;p$previous_lnhr<-z$lnhr
out<-do.call(rbind,lapply(ids,function(id){
  d<-p[p$id==id,];stopifnot(nrow(d)==9,sqrt(var(d$lnhr)*8/9)>1e-12,sqrt(var(d$previous_lnhr)*8/9)>1e-12)
  data.frame(id=id,pairs=nrow(d),correlation=cor(d$lnhr,d$previous_lnhr))
}))
cols<-c('source_row','lnhr','lnwg','kids','ageh','agesq','disab','id','year','previous_source_row','previous_year','previous_lnhr')
dir.create(a[2]);write.table(as.data.frame(lapply(p[cols],function(x)sprintf('%.17g',x))),file.path(a[2],'pairs.csv'),sep=',',row.names=FALSE,quote=FALSE)
write.table(as.data.frame(lapply(out,function(x)sprintf('%.17g',x))),file.path(a[2],'correlations.csv'),sep=',',row.names=FALSE,quote=FALSE)
cat(R.version.string,'N pairs',nrow(p),'people',G,'\n')
