# Development-only profile/quadrature reference, independent of native Newton/formulas.
# Rscript gompertz_reference.R duration.csv NEW_DIRECTORY [original_DTA_or_-] [noconstant]
args<-commandArgs(TRUE);stopifnot(length(args)%in%2:4,!file.exists(args[2]))
no_intercept<-length(args)==4
if(no_intercept)stopifnot(args[4]=='noconstant')
if(length(args)>=3 && args[3]!='-') {
  raw<-foreign::read.dta(args[3],convert.factors=FALSE)
  raw<-data.frame(source_row=seq_len(nrow(raw)),raw,check.names=FALSE)
} else raw<-read.csv(args[1],check.names=FALSE)
d<-raw;d$rr_ui<-d$reprate*d$ui;d$dr_ui<-d$disrate*d$ui
x<-strsplit('reprate disrate ui rr_ui dr_ui logwage tenure slack abolpos explose stateur houshead married female child ychild nonwhite age schlt12 schgt12 smsa bluecoll mining constr transp trade fire services pubadmin year85 year87 year89 midatl encen wncen southatl escen wscen mountain pacific',' ')[[1]]
stopifnot(nrow(d)>42,all(is.finite(as.matrix(d))),all(d$spell>0),sum(d$censor1)>0,
 all(as.matrix(d[paste0('censor',1:4)])%in%c(0,1)),all(rowSums(d[paste0('censor',1:4)])<=1),all(d$ui%in%c(0,1)),!anyDuplicated(d$source_row))
X<-as.matrix(d[x]);if(!no_intercept)X<-cbind(intercept=1,X)
n<-nrow(X);k<-ncol(X);D<-d$censor1;time<-d$spell;time_scale<-max(time);tau<-time/time_scale
# Numerical integration of exp(z*u) supplies independent log zeroth moment and
# moment ratios; scaled integrands never exponentiate a large positive argument.
integrated_moments<-function(z) {
  maximum<-max(z,0)
  m<-vapply(0:2,function(j)integrate(function(u)u^j*exp(z*u-maximum),0,1,
    rel.tol=1e-12,abs.tol=1e-300,subdivisions=200)$value,numeric(1))
  stopifnot(all(is.finite(m)),all(m>0))
  c(log_m0=maximum+log(m[1]),mean_u=m[2]/m[1],mean_u2=m[3]/m[1])
}
row_moments<-function(q) {
  times<-sort(unique(tau))
  m<-t(vapply(times,function(t)integrated_moments(q*t),numeric(3)))
  m[match(tau,times),,drop=FALSE]
}
# Conditional beta maximization is exactly a Poisson GLM algebraic subproblem,
# NOT a Poisson duration DGP. A scalar profile score is the outer optimization.
solve_profile<-function(design) {
  p<-ncol(design)
  evaluate<-function(q,retain=FALSE) {
    M<-row_moments(q);offset<-log(time)+M[,1]
    if(p>0) {
      glm<-glm.fit(design,D,family=poisson(),offset=offset,
        control=glm.control(epsilon=1e-13,maxit=200))
      stopifnot(glm$converged,glm$rank==p,all(is.finite(glm$coefficients)))
      beta<-glm$coefficients;eta<-drop(design%*%beta)
    } else {beta<-numeric();eta<-rep(0,n)}
    H<-exp(eta+offset);score_q<-sum(tau*(D-H*M[,2]))
    if(!retain)return(score_q)
    U<-cbind(design*(D-H),tau*(D-H*M[,2]))
    Z<-cbind(design,tau*M[,2]);I<-crossprod(Z*sqrt(H))
    I[p+1,p+1]<-sum(H*tau^2*M[,3])
    list(beta=beta,q=q,H=H,eta=eta,U=U,I=I,LL=sum(D*(eta+q*tau)-H))
  }
  lo<--1;hi<-1
  while(evaluate(lo)<0){lo<-lo*2;stopifnot(lo>=-512)}
  while(evaluate(hi)>0){hi<-hi*2;stopifnot(hi<=512)}
  q<-uniroot(evaluate,c(lo,hi),tol=1e-13,maxiter=1000)$root
  result<-evaluate(q,TRUE);bread<-solve(result$I)
  stopifnot(max(abs(colSums(result$U)))<1e-7,
    max(abs(result$I%*%bread-diag(p+1)))<1e-8)
  chol(result$I)
  result$bread<-bread;result
}
s<-solve_profile(X)
null<-solve_profile(if(no_intercept)matrix(numeric(),nrow=n,ncol=0)else matrix(1,nrow=n,ncol=1))
beta<-s$beta;gamma<-s$q/time_scale
transform<-diag(c(rep(1,k),1/time_scale))
model_v<-transform%*%s$bread%*%transform
robust_v<-transform%*%s$bread%*%crossprod(s$U)%*%s$bread%*%transform*n/(n-1)
# Finite differences of the independent integrated gradient certify ALL joint
# observed-information blocks; the bound covers numerical differencing error.
objective<-function(p) {
  M<-row_moments(p[k+1]);eta<-drop(X%*%p[seq_len(k)])
  -sum(D*(eta+p[k+1]*tau)-exp(eta+log(time)+M[,1]))
}
gradient<-function(p) {
  M<-row_moments(p[k+1]);eta<-drop(X%*%p[seq_len(k)]);H<-exp(eta+log(time)+M[,1])
  -c(drop(crossprod(X,D-H)),sum(tau*(D-H*M[,2])))
}
numeric_I<-optimHess(c(beta,s$q),objective,gradient,control=list(ndeps=rep(1e-5,k+1)))
information_discrepancy<-max(abs(numeric_I-s$I)/(1+abs(s$I)))
stopifnot(information_discrepancy<1e-6,s$LL>=null$LL-1e-9)
slope<-if(no_intercept)seq_len(k)else 2:k;df<-length(slope)
constant<-sum(D*log(time));z95<-qnorm(.975)
gates_for<-function(V,robust) {
  gates<-data.frame(path=character(),value=numeric())
  add<-function(path,value){stopifnot(length(value)==1,is.finite(value));gates[nrow(gates)+1,]<<-list(path,unname(value))}
  se<-sqrt(diag(V));z<-c(beta,gamma)/se
  add('/observation_count',n);add('/failure_count',sum(D));add('/censored_count',n-sum(D))
  add('/covariance_correction',if(robust)n/(n-1)else 1)
  add('/log_likelihood',s$LL);add('/normalized_log_likelihood',s$LL+constant)
  add('/null_log_likelihood',null$LL);add('/null_normalized_log_likelihood',null$LL+constant)
  lr<-2*(s$LL-null$LL);add('/likelihood_ratio',lr);add('/likelihood_ratio_df',df)
  if(!robust)add('/likelihood_ratio_p_value',pchisq(lr,df,lower.tail=FALSE))
  wald<-drop(crossprod(beta[slope],solve(V[slope,slope],beta[slope])))
  add('/omnibus_wald/statistic',wald);add('/omnibus_wald/degrees_of_freedom',df)
  add('/omnibus_wald/p_value',pchisq(wald,df,lower.tail=FALSE))
  for(i in seq_len(k+1)) {
    values<-c(value=c(beta,gamma)[i],standard_error=se[i],statistic=z[i],
      p_value=2*pnorm(-abs(z[i])),confidence_lower=c(beta,gamma)[i]-z95*se[i],confidence_upper=c(beta,gamma)[i]+z95*se[i])
    names(values)<-c('value','standard_error','statistic','p_value','confidence_lower','confidence_upper')
    for(field in names(values))add(if(i<=k)paste0('/coefficients/',i-1,'/',field)else paste0('/shape/',field),values[field])
  }
  for(i in seq_len(k+1))for(j in seq_len(k+1))add(paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
  gates
}
point<-function(data,prefix) {
  design<-as.matrix(data[x]);if(!no_intercept)design<-cbind(intercept=1,design)
  eta<-drop(design%*%beta)
  # Direct numerical integration, not the proposed native exprel expressions.
  integral<-vapply(data$spell,function(t)exp(integrated_moments(gamma*t)[1])*t,numeric(1))
  H<-exp(eta)*integral
  result<-data.frame(xb=eta,hazard=exp(eta+gamma*data$spell),cumulative_hazard=H,net_survival=exp(-H))
  names(result)<-paste0(prefix,names(result));result
}
ui0<-d;ui0$ui<-0;ui0$rr_ui<-0;ui0$dr_ui<-0
ui1<-d;ui1$ui<-1;ui1$rr_ui<-ui1$reprate;ui1$dr_ui<-ui1$disrate
pred<-cbind(raw,point(d,''),point(ui0,'ui0_'),point(ui1,'ui1_'))
model_gates<-gates_for(model_v,FALSE);robust_gates<-gates_for(robust_v,TRUE)
stopifnot(dir.create(args[2]))
write_numeric<-function(data,name)write.table(as.data.frame(lapply(data,function(v)
  if(is.numeric(v))sprintf('%.17g',v)else v),check.names=FALSE),file.path(args[2],name),sep=',',quote=FALSE,row.names=FALSE)
write_numeric(model_gates,'model_based.csv');write_numeric(robust_gates,'robust.csv')
write_numeric(pred,'predictions.csv')
write.table(as.data.frame(lapply(raw,function(v)sprintf('%a',v)),check.names=FALSE),
  file.path(args[2],'source_hex.csv'),sep=',',quote=FALSE,row.names=FALSE)
writeLines(c(colnames(X),'gamma'),file.path(args[2],'parameter_names.txt'))
# Independent quadrature fixtures exercise zero continuity, both signs and tails.
z<-c(-1000,-100,-10,-2,-1,-.1,-1e-8,0,1e-8,.1,1,2,10,100,1000)
helper<-cbind(data.frame(z=z),as.data.frame(t(vapply(z,integrated_moments,numeric(3)))))
write_numeric(helper,'moments.csv')
cat(R.version.string,'N',n,'P',k+1,'gamma',format(gamma,digits=17),
  'model gates',nrow(model_gates),'robust gates',nrow(robust_gates),
  'max score',max(abs(colSums(s$U))),'numeric information relative',information_discrepancy,'\n')
