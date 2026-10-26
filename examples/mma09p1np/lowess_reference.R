# Independent per-window base-R weighted QR, NOT stats::lowess's different windows.
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)%in%c(2L,3L),!dir.exists(args[2]))
options(digits=17)
raw <- read.csv(args[1],check.names=FALSE)
if (length(args)==3L) {
  fields<-c('intnum','persnum','age','educatn','earnings','hours','sex','kids','married')
  original<-read.fwf(args[3],widths=c(4,3,2,2,6,4,1,2,1),col.names=fields,
    colClasses='numeric',na.strings=c('','.'),strip.white=TRUE)
  original<-data.frame(source_row=seq_len(nrow(original)),original)
  stopifnot(identical(names(raw),names(original)),identical(is.na(raw),is.na(original)),
    all(as.matrix(raw)==as.matrix(original),na.rm=TRUE))
  raw<-original
}
source_input<-all(c('educatn','earnings','hours','age')%in%names(raw))
if (source_input) {
  sample<-raw[!is.na(raw$educatn)&raw$educatn%in%1:97&raw$earnings>0&raw$age==36,]
  sample<-sample[order(sample$source_row),]
  sample$capped_earnings<-pmin(sample$earnings,100000)
  sample$hwage<-sample$capped_earnings/sample$hours
  sample$lnhwage<-log(sample$hwage)
  input<-data.frame(source_row=sample$source_row,x=sample$educatn,y=sample$lnhwage)
} else {
  stopifnot(identical(names(raw),c('source_row','x','y')))
  input<-raw
}
stopifnot(!anyDuplicated(input$source_row))
centered <- function(x,y,span) {
  n<-length(x)
  stopifnot(n==length(y),n>=2,n<=10000,all(is.finite(x)),all(is.finite(y)),
    span>0,span<1,n*span>=.5)
  order<-order(x,seq_len(n));xx<-x[order];yy<-y[order]
  k<-floor((n*span-.5)/2)
  stopifnot(n*min(n,2*k+1)<=20000000)
  fitted<-numeric(n);constant<-0;score<-0
  for(i in seq_len(n)) {
    at<-seq.int(max(1,i-k),min(n,i+k))
    # An unrelated distant response cannot determine this window's numeric scale.
    scale<-max(abs(yy[at]));if(scale==0)scale<-1
    d<-max(abs(xx[at]-xx[i]));stopifnot(is.finite(d))
    z<-if(d==0)rep(0,length(at))else(xx[at]-xx[i])/d
    w<-(1-(abs(z)/1.0001)^3)^3
    design<-if(d==0)matrix(1,length(at),1)else cbind(1,z)
    # Radius normalization and positive inflated-endpoint weights give a second
    # QR-column scale >=~5e-8 relative to the intercept under N<=10000.
    # tol1e-12 avoids dropping an identified local slope; exact radius0 alone
    # deliberately uses an intercept-only reference model.
    fit<-lm.wfit(design,yy[at]/scale,w,tol=1e-12)
    stopifnot(fit$rank==ncol(design),all(is.finite(fit$coefficients)))
    fitted[i]<-fit$coefficients[1]*scale
    score<-max(score,abs(crossprod(design,w*fit$residuals))/length(at))
    constant<-constant+as.integer(d==0)
  }
  result<-numeric(n);result[order]<-fitted
  stopifnot(all(is.finite(result)),score<1e-12)
  list(values=result,half_window=k,local_constant_windows=constant,max_scaled_score=score)
}
# Scale-relative locality regression: an absolute tolerance would accept zero
# after a shared erroneous global normalization in both native and QR paths.
for (x in list(rep(0,7),0:6)) {
  control<-centered(x,c(rep(1e-100,6),1e308),.4)$values[1:5]
  stopifnot(all(control!=0),all(abs(control/1e-100-1)<1e-12))
}
spans<-c(.8,.4,.1);names<-c('lowess_08','lowess_04','lowess_01')
fits<-lapply(spans,function(span)centered(input$x,input$y,span))
fitted<-input
for(i in seq_along(fits))fitted[[names[i]]]<-fits[[i]]$values
metadata<-do.call(rbind,lapply(seq_along(fits),function(i)data.frame(
  bandwidth=spans[i],observations=nrow(input),half_window=fits[[i]]$half_window,
  local_constant_windows=fits[[i]]$local_constant_windows,max_scaled_score=fits[[i]]$max_scaled_score)))
# An explicit non-reference counterexample prevents calling R's default window
# implementation our oracle, even when robust iterations/interpolation are off.
comparison<-do.call(rbind,lapply(seq_along(fits),function(i) {
  nearest<-stats::lowess(input$x,input$y,f=spans[i],iter=0,delta=0)
  values<-nearest$y[match(input$x,nearest$x)]
  data.frame(bandwidth=spans[i],max_difference=max(abs(values-fits[[i]]$values)))
}))
dir.create(args[2])
write_precise<-function(frame,name) {
  text<-as.data.frame(lapply(frame,function(z)ifelse(is.na(z),'',sprintf('%.17g',z))),check.names=FALSE)
  write.table(text,file.path(args[2],name),sep=',',row.names=FALSE,quote=FALSE,na='')
}
write_precise(input,'input.csv');write_precise(fitted,'fitted.csv')
write_precise(metadata,'metadata.csv');write_precise(comparison,'nearest_window_difference.csv')
if(source_input){write_precise(raw,'source.csv');write_precise(sample,'sample.csv')}
writeLines(c(R.version.string,paste('stats',packageVersion('stats'))),file.path(args[2],'versions.txt'))
cat(nrow(input),'rows,',3*nrow(input),'QR fitted values; all normal-equation certificates<1e-12\n')
