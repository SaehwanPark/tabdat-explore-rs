# Development-only original-MOM marginal likelihood/information reference.
# Rscript random_effects_ml_probe.R /path/to/MOM.dat [float32|float64] [output-dir]
options(digits=17)
library(nlme)
args <- commandArgs(trailingOnly=TRUE); stopifnot(length(args) %in% 1:3)
d <- as.data.frame(matrix(scan(args[1],quiet=TRUE),ncol=8,byrow=TRUE)); names(d)<-c('y','x','kids','ageh','agesq','disab','id','time')
stopifnot(nrow(d)==5320L,identical(d$id,rep(as.numeric(1:532),each=10)),identical(d$time,rep(as.numeric(1979:1988),532)))
if(length(args)>=2L) stopifnot(args[2] %in% c('float32','float64'))
if(length(args)>=2L && args[2]=='float32') {
  f32<-function(x)readBin(writeBin(as.double(x),raw(),size=4),double(),n=length(x),size=4)
  for(name in names(d)) d[[name]]<-f32(d[[name]])
}
X<-model.matrix(~x,d); y<-d$y; T<-10; G<-532; N<-nrow(d); K<-ncol(X)
Xm<-apply(X,2,function(x)ave(x,d$id)); ym<-ave(y,d$id)
Xw<-X-Xm; yw<-y-ym
A<-crossprod(Xw); B<-crossprod(Xm); c<-crossprod(Xw,yw); f<-crossprod(Xm,ym)
profile<-function(eta,details=FALSE) {
 q<-exp(-eta); beta<-unname(drop(solve(A+q*B,c+q*f))); ew<-drop(yw-Xw%*%beta); eb<-drop(ym-Xm%*%beta)
 a<-(sum(ew^2)+q*sum(eb^2))/N; v<-a/q
 ll<--.5*(N*(log(2*pi)+1+log(a))+G*eta)
 if(details) list(beta=beta,a=a,v=v,ll=ll,ew=ew,eb=eb) else -ll
}
eta<-uniroot(function(eta){ r<-profile(eta,TRUE); .5*(sum(r$eb^2)/r$v-G) },c(0,24),tol=1e-12)$root
opt<-list(minimum=eta); r<-profile(eta,TRUE)
H<-matrix(0,K+2,K+2); H[1:K,1:K]<-A/r$a+B/r$v
H[1:K,K+1]<-crossprod(Xw,r$ew)/r$a; H[K+1,1:K]<-H[1:K,K+1]
H[1:K,K+2]<-crossprod(Xm,r$eb)/r$v; H[K+2,1:K]<-H[1:K,K+2]
H[K+1,K+1]<-sum(r$ew^2)/(2*r$a);H[K+2,K+2]<-sum(r$eb^2)/(2*r$v)
V<-solve(H); print(c(beta=r$beta,a=r$a,b=(r$v-r$a)/T,loglik=r$ll,eta=opt$minimum));print(sqrt(diag(V))[1:K])
su<-sqrt((r$v-r$a)/T); se<-sqrt(r$a); gu<-c(rep(0,K),-r$a/(2*T*su),r$v/(2*T*su));ge<-c(rep(0,K),.5*se,0)
rho<-(r$v-r$a)/(r$v+(T-1)*r$a); gr<-c(rep(0,K),-T*r$a*r$v/(r$v+(T-1)*r$a)^2,T*r$a*r$v/(r$v+(T-1)*r$a)^2)
print(c(sigma_u=su,se_sigma_u=sqrt(drop(t(gu)%*%V%*%gu)),sigma_e=se,se_sigma_e=sqrt(drop(t(ge)%*%V%*%ge)),rho=rho,se_rho=sqrt(drop(t(gr)%*%V%*%gr))))
fit<-lme(y~x,random=~1|id,data=d,method='ML',control=lmeControl(tolerance=1e-10,msTol=1e-10));print(c(fixef(fit),sigma=fit$sigma,loglik=as.numeric(logLik(fit))));print(VarCorr(fit));print(sqrt(diag(vcov(fit))))

nullsw<-sum((y-ym)^2); nullsb<-sum((ym-mean(y))^2)
nulla<-nullsw/(N-G); nullv<-nullsb/G
stopifnot(nullv>nulla)
nullll<--.5*((N-G)*(log(2*pi)+1+log(nulla))+G*(log(2*pi)+1+log(nullv)))
cat('Restricted null loglik =',nullll,'; slope LR =',2*(r$ll-nullll),'\n')

stopifnot(max(abs(fixef(fit)-r$beta))<1e-6,abs(as.numeric(logLik(fit))-r$ll)<1e-8)
if(length(args)==3L) {
  dir.create(args[3],recursive=TRUE,showWarnings=FALSE)
  rows <- list()
  emit <- function(path,value) rows[[length(rows)+1L]] <<- data.frame(path,value=unname(value))
  for(i in 1:K) {
    se <- sqrt(V[i,i]); z <- r$beta[i]/se
    values <- c(value=r$beta[i],standard_error=se,statistic=z,p_value=2*pnorm(-abs(z)),ci_lower=r$beta[i]-qnorm(.975)*se,ci_upper=r$beta[i]+qnorm(.975)*se)
    for(name in names(values)) emit(paste0('/coefficients/',i-1,'/',name),values[[name]])
    for(j in 1:K) emit(paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
  }
  for(i in 1:(K+2)) for(j in 1:(K+2)) emit(paste0('/ml/joint_covariance/',i-1,'/',j-1),V[i,j])
  wald <- drop(t(r$beta[-1])%*%solve(V[-c(K+1,K+2,1),-c(K+1,K+2,1),drop=FALSE],r$beta[-1]))
  vals <- c(sigma_e_squared=r$a,sigma_u_squared=(r$v-r$a)/T,theta=1-exp(-eta/2),rho=rho,log_likelihood=r$ll,wald_statistic=wald,wald_p_value=pchisq(wald,K-1,lower.tail=FALSE))
  for(name in names(vals)) emit(paste0('/',name),vals[[name]])
  vals <- c(eta=eta,log_likelihood=r$ll,null_log_likelihood=nullll,null_sigma_e_squared=nulla,null_sigma_u_squared=(nullv-nulla)/T,likelihood_ratio=2*(r$ll-nullll),likelihood_ratio_p_value=pchisq(2*(r$ll-nullll),K-1,lower.tail=FALSE))
  for(name in names(vals)) emit(paste0('/ml/',name),vals[[name]])
  write.csv(do.call(rbind,rows),file.path(args[3],'reference.csv'),row.names=FALSE,quote=FALSE)
  output <- d; names(output)<-c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
  output <- cbind(source_row=seq_len(N),output)
  output$ml_xb <- drop(X%*%r$beta); output$ml_residual <- y-output$ml_xb
  write.csv(output,file.path(args[3],'predictions.csv'),row.names=FALSE,quote=FALSE)
}
