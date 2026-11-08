# Independent direct improper integration of complete-data likelihood ratios.
# No native moments/coefficients, Gauss-Hermite nodes or Stein score as inputs.
a<-commandArgs(TRUE);stopifnot(length(a)==1,!dir.exists(a[1]));dir.create(a[1],recursive=TRUE)
inputs<-file(file.path(a[1],'posterior_input.csv'),'wx');refs<-file(file.path(a[1],'posterior.csv'),'wx')
writeLines('case_id,row,y,mu,delta_eta,t,delta_t',inputs)
writeLines('case_id,linear,log_ratio',refs)
base<-data.frame(S=c(0,2,100,100000,0,3),M=c(.2,3.2,8.2,10000,100,1e-5),t=c(log(.3),log(.8),log(2),0,.99,2))
case<-0L;max_error<-0
for(k in seq_len(nrow(base)))for(step in c(.001,-.001,1e-12)){
 case<-case+1L;n<-c(1L,3L,5L)[(k-1L)%%3L+1L]
 y<-numeric(n);if(n==1L)y[1]<-base$S[k] else {y[1]<-floor(base$S[k]/3);y[2]<-base$S[k]-y[1]}
 mu<-base$M[k]*seq_len(n)/sum(seq_len(n));de<-step*(.2+.35*(seq_len(n)-(n+1)/2));dt<--.7*step
 S<-sum(y);M<-sum(mu);v<-exp(base$t[k]);center<-uniroot(function(u)S-M*exp(u)-u/v,c(-60,60),tol=2e-14)$root
 lambda<-M*exp(center);curvature<-lambda+1/v
 yd<-sum(y*de);md<-sum(mu*de);me<-sum(mu*expm1(de));prior<-expm1(-dt)
 # Integrate in centered curvature units; ratios are formed inside the integral,
 # never by subtracting two large marginal log-likelihood values.
 integrand<-function(z,kind)vapply(z,function(zz){
  du<-zz/sqrt(curvature);u<-center+du
  logw<-(S-lambda-center/v)*du-lambda*(expm1(du)-du)-du^2/(2*v)
  if(!is.finite(logw)||logw< -740)return(0)
  density<-exp(logw)
  if(kind=='mass')return(density)
  if(kind=='linear'){
   value<-yd-exp(center)*md-exp(center)*md*expm1(du)+dt*(u^2/(2*v)-.5)
   return(density*value)
  }
  h<-yd-exp(center)*me-dt/2-center^2/(2*v)*prior-exp(center)*me*expm1(du)-du*(2*center+du)/(2*v)*prior
  if(abs(h)<.5)return(density*expm1(h))
  exp(logw+h)-density
 },0.)
 integral<-function(kind){
  r<-integrate(function(z)integrand(z,kind),-Inf,Inf,rel.tol=2e-12,abs.tol=if(kind=='mass')1e-14 else 1e-30,subdivisions=4000L)
  stopifnot(r$message=='OK',is.finite(r$value));max_error<<-max(max_error,r$abs.error);r$value
 }
 mass<-integral('mass');linear<-integral('linear')/mass;ratio<-log1p(integral('ratio')/mass)
 for(i in seq_len(n))writeLines(sprintf('%d,%d,%.17g,%.17g,%.17g,%.17g,%.17g',case,i,y[i],mu[i],de[i],base$t[k],dt),inputs)
 writeLines(sprintf('%d,%.17g,%.17g',case,linear,ratio),refs)
}
close(inputs);close(refs)
cat(R.version.string,';18 direct improper-integral cases; max reported quadrature abs.error',format(max_error,digits=17),'\n')
