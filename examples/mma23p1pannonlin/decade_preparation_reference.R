# Development oracle: raw-token indexing, never native/SQL derived values.
options(digits=17)
a<-commandArgs(TRUE);stopifnot(length(a)==2,!file.exists(a[2]))
tokens<-scan(a[1],what=character(),quiet=TRUE);stopifnot(length(tokens)%%25==0)
raw<-matrix(tokens,ncol=25,byrow=TRUE);g<-nrow(raw);stopifnot(g>=2,g<=346)
stopifnot(all(which(raw=='.',arr.ind=TRUE)[,2]==2L))
v<-suppressWarnings(matrix(as.numeric(raw),ncol=25));stopifnot(all(is.finite(v[,-2])),all(is.na(v[,2])|is.finite(v[,2])))
stopifnot(all(v[,16:25]>=0),all(v[,16:25]<=1e9),all(v[,16:25]==floor(v[,16:25])),all(abs(v[,6:15])<=40))
records<-lapply(seq_len(g),function(i){
 history<-v[i,6:15];pat<-v[i,16:25]
 d<-data.frame(id=i,year=1970:1979,cusip=v[i,1],ardssic=v[i,2],scisect=v[i,3],logk=v[i,4],sumpat=v[i,5],logr=history,pat=pat,
   lpat=log(ifelse(pat==0,.5,pat)),dpat=as.integer(pat>0),randd=exp(history))
 for(j in 1:5)d[[paste0('logrl',j)]]<-c(rep(NA_real_,j),history[seq_len(10-j)])
 for(j in 2:5)d[[paste0('dyear',j)]]<-as.integer(d$year==1974+j)
 d
})
d<-do.call(rbind,records);selected<-d[d$year>=1975,]
exact_csv<-function(d,path){
 out<-as.data.frame(lapply(d,function(x)ifelse(is.na(x),'',sprintf('%.17g',x))),stringsAsFactors=FALSE)
 write.table(out,path,sep=',',row.names=FALSE,col.names=TRUE,quote=FALSE,na='')
}
summary_rows<-function(d,cohort)do.call(rbind,lapply(names(d),function(name){
 x<-d[[name]];x<-x[is.finite(x)];n<-length(x)
 data.frame(cohort=cohort,variable=name,count=n,mean=if(n)mean(x) else NA_real_,std_dev=if(n>1)sd(x) else NA_real_,minimum=if(n)min(x) else NA_real_,maximum=if(n)max(x) else NA_real_)
}))
dir.create(a[2]);exact_csv(d,file.path(a[2],'full.csv'));exact_csv(selected,file.path(a[2],'selected.csv'))
s<-rbind(summary_rows(d,'full'),summary_rows(selected,'selected'))
for(name in names(s)[-(1:2)])s[[name]]<-ifelse(is.na(s[[name]]),'',sprintf('%.17g',s[[name]]))
write.table(s,file.path(a[2],'summaries.csv'),sep=',',row.names=FALSE,col.names=TRUE,quote=FALSE,na='')
cat(R.version.string,';',nrow(d),'full/',nrow(selected),'selected records; direct raw history; variable-wise N and N-1 SD\n')
