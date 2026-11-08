# Independent development reference: adaptive improper integration, not GH nodes.
options(digits=17)
args<-commandArgs(trailingOnly=TRUE);stopifnot(length(args)==1L,!file.exists(args[1]))
cases<-data.frame(total=c(0,2,100,100000,0,3),mean_sum=c(.2,3.2,8.2,10000,100,1e-5),log_variance=c(log(.3),log(.8),log(2),0,.99,2))
evaluate<-function(s,m,t){
  variance<-exp(t)
  mode<-uniroot(function(u)s-m*exp(u)-u/variance,c(-60,60),tol=1e-14)$root
  scale<-1/sqrt(m*exp(mode)+1/variance)
  log_kernel<-function(u)s*u-m*exp(u)-u*u/(2*variance)
  anchor<-log_kernel(mode)
  density<-function(z){
    u<-mode+scale*z;logw<-log_kernel(u)-anchor
    logw[!is.finite(logw)]<- -Inf
    exp(logw)
  }
  integral<-function(f){
    integrate(function(z){
      w<-density(z);out<-numeric(length(z));positive<-w>0
      # Never multiply an infinite tail moment by an underflowed zero density.
      out[positive]<-w[positive]*f(mode+scale*z[positive]);out
    },-Inf,Inf,rel.tol=1e-12,abs.tol=1e-12,subdivisions=2000L,stop.on.error=TRUE)$value
  }
  mass<-integral(function(u)rep(1,length(u)))
  e<-integral(exp)/mass;u2<-integral(function(u)u*u)/mass;mean_u<-integral(identity)/mass
  c(log_integral=log(scale)+anchor+log(mass)-log(2*pi*variance)/2,
    mean_exp_u=e,mean_u2=u2,var_exp_u=integral(function(u)(exp(u)-e)^2)/mass,
    cov_exp_u_u2=integral(function(u)(exp(u)-e)*(u*u-u2))/mass,
    var_u2=integral(function(u)(u*u-u2)^2)/mass,
    mean_u=mean_u,cov_exp_u_u=integral(function(u)(exp(u)-e)*(u-mean_u))/mass)
}
values<-t(vapply(seq_len(nrow(cases)),function(i)evaluate(cases$total[i],cases$mean_sum[i],cases$log_variance[i]),numeric(8)))
write.csv(cbind(cases,values),args[1],row.names=FALSE,quote=FALSE)
cat(R.version.string,'; six direct adaptive improper-integral posterior controls\n')
