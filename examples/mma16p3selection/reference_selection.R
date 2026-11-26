# Development-only R reference for the March 2005 MMA §16.6 RAND program.
# Base R only; this is a separate likelihood/optimizer and is not a runtime dependency.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=3L) stop("usage: reference_selection.R source.asc quantities.csv predictions.csv")
if (file.exists(args[2]) || file.exists(args[3])) stop("reference outputs already exist")
tokens <- scan(args[1], what=character(), quiet=TRUE)
if (length(tokens)!=111480L || length(tokens) %% 20L != 0L) stop("expected the 5,574 x 20 prepared ASCII token stream")
raw <- matrix(tokens, ncol=20L, byrow=TRUE)
names <- c("DMED","MED","LNMED","LC","IDP","LPI","FMDE","PHYSLIM","NDISEASE","HLTHG","HLTHF","HLTHP","LINC","LFAM","EDUCDEC","AGE","FEMALE","CHILD","FEMCHILD","BLACK")
colnames(raw) <- names
d <- as.numeric(raw[,"DMED"])
dat <- as.data.frame(apply(raw,2L,function(column) suppressWarnings(ifelse(column==".",NA_real_,as.numeric(column)))))
names(dat) <- names
predictors <- names[4:20]
if (nrow(dat)!=5574L || sum(d==1)!=4281L || sum(d==0)!=1293L ||
    sum(is.na(dat$LNMED))!=1293L || any(dat$DMED != as.numeric(dat$MED>0),na.rm=TRUE) ||
    any(is.na(dat$LNMED)!=(dat$DMED==0))) stop("unexpected sample or source missingness")
selection <- dat$DMED
selected <- which(selection==1)
y <- dat$LNMED[selected]
X <- cbind(intercept=1,as.matrix(dat[selected,predictors,drop=FALSE]))
X_all <- cbind(intercept=1,as.matrix(dat[,predictors,drop=FALSE]))
Z <- cbind(intercept=1,as.matrix(dat[,predictors,drop=FALSE]))
n <- nrow(dat); n1 <- length(selected); p <- ncol(X); q <- ncol(Z)

log_phi <- function(x) dnorm(x,log=TRUE)
inverse_mills <- function(x) exp(log_phi(x)-pnorm(x,log.p=TRUE))
probit_eval <- function(gamma) {
  index <- drop(Z%*%gamma)
  pos <- inverse_mills(index)
  neg <- inverse_mills(-index)
  curvature <- ifelse(selection==1,pos*(pos+index),neg*(neg-index))
  signed <- ifelse(selection==1,pos,-neg)
  list(ll=sum(ifelse(selection==1,pnorm(index,log.p=TRUE),pnorm(-index,log.p=TRUE))),
       score=drop(crossprod(Z,signed)),
       information=crossprod(Z,curvature*Z),index=index)
}
glm0 <- glm.fit(x=Z,y=selection,family=binomial(link="probit"),
                control=glm.control(epsilon=1e-13,maxit=100))
gamma <- unname(glm0$coefficients)
for (iteration in 1:40) {
  fit <- probit_eval(gamma)
  step <- solve(fit$information,fit$score)
  candidate <- gamma+step
  if (probit_eval(candidate)$ll < fit$ll) stop("base-R observed-Newton probit step reduced likelihood")
  gamma <- candidate
  if (max(abs(fit$score))/n<1e-12 && max(abs(step)/(1+abs(gamma)))<1e-12) break
}
probit <- probit_eval(gamma)
probit_covariance <- solve(probit$information)
selection_index <- probit$index
selection_probability <- pnorm(selection_index)
mills <- inverse_mills(selection_index)
delta <- mills*(mills+selection_index)

# Two-part positive-probability probit and selected-sample Gaussian OLS.
ols <- lm.fit(x=X,y=y)
ols_sse <- sum(ols$residuals^2)
ols_df <- n1-p
ols_sigma2 <- ols_sse/ols_df
ols_covariance <- ols_sigma2*solve(crossprod(X))
ols_loglik <- -n1/2*(log(2*pi)+1+log(ols_sse/n1))

# Existing two-step estimator and its separately corrected generated-regressor covariance.
W <- cbind(X,mills_coefficient=mills[selected])
selected_z <- Z[selected,,drop=FALSE]
two_step_ols <- lm.fit(x=W,y=y)
two_step_beta <- unname(two_step_ols$coefficients)
alpha <- tail(two_step_beta,1L)
two_step_sse <- sum(two_step_ols$residuals^2)
sigma_two_step <- sqrt(two_step_sse/n1+alpha^2*mean(delta[selected]))
rho_two_step <- alpha/sigma_two_step
A_inv <- solve(crossprod(W))
B <- crossprod(W,delta[selected]*selected_z)
two_step_covariance <- A_inv%*%(crossprod(W,(sigma_two_step^2-alpha^2*delta[selected])*W)+
  alpha^2*B%*%probit_covariance%*%t(B))%*%A_inv

# Joint observed-data Heckman likelihood in [beta,gamma,lnsigma,athrho] order.
y_scale <- sqrt(mean((y-mean(y))^2))
scales <- c(y_scale/sqrt(colMeans(X^2)),1/sqrt(colMeans(Z^2)),1,1)
two_step_mean <- drop(W%*%two_step_beta)
two_step_residual <- y-two_step_mean
sigma_start <- sqrt(two_step_sse/n1+alpha^2*mean(delta[selected]))
start <- c(two_step_beta[seq_len(p)],gamma,log(sigma_start),atanh(alpha/sigma_start))
loglik_score <- function(theta) {
  beta <- theta[seq_len(p)]
  gamma <- theta[p+seq_len(q)]
  log_sigma <- theta[p+q+1L]
  atanh_rho <- theta[p+q+2L]
  if (!is.finite(log_sigma) || abs(atanh_rho)>8) stop("parameter outside regular range")
  sigma <- exp(log_sigma); rho <- tanh(atanh_rho); c0 <- sqrt(1-rho^2)
  e <- (y-drop(X%*%beta))/sigma
  index <- drop(Z%*%gamma)
  conditional <- (index[selected]+rho*e)/c0
  conditional_mills <- inverse_mills(conditional)
  ll <- sum(dnorm(e,log=TRUE)-log_sigma+pnorm(conditional,log.p=TRUE))+
    sum(pnorm(-index[selection==0],log.p=TRUE))
  derivative_e <- -e+conditional_mills*rho/c0
  score_beta <- drop(crossprod(X,-derivative_e/sigma))
  score_gamma <- drop(crossprod(Z[selection==0,,drop=FALSE],-inverse_mills(-index[selection==0])))+
    drop(crossprod(Z[selected,,drop=FALSE],conditional_mills/c0))
  score_log_sigma <- sum(-1-derivative_e*e)
  score_athrho <- sum(conditional_mills*(e+rho*index[selected])/c0)
  list(ll=ll,score=c(score_beta,score_gamma,score_log_sigma,score_athrho))
}
objective <- function(psi) -loglik_score(psi*scales)$ll/n
gradient <- function(psi) -loglik_score(psi*scales)$score*scales/n
ml <- optim(start/scales,objective,gr=gradient,method="BFGS",
            control=list(maxit=500,reltol=1e-14))
if (ml$convergence!=0L) stop("base-R BFGS did not converge")
psi <- ml$par
theta <- psi*scales
ml_fit <- loglik_score(theta)
if (max(abs(ml_fit$score*scales))/n>2e-7) stop("base-R joint score exceeds the reference gate")
information <- matrix(0,length(theta),length(theta))
for (j in seq_along(psi)) {
  h <- 2e-5*(1+abs(psi[j])); plus <- minus <- psi
  plus[j] <- plus[j]+h; minus[j] <- minus[j]-h
  plus_score <- loglik_score(plus*scales)$score*scales
  minus_score <- loglik_score(minus*scales)$score*scales
  information[,j] <- -(plus_score-minus_score)/(2*h)
}
information <- (information+t(information))/2
covariance_scaled <- solve(information)
ml_covariance <- diag(scales)%*%covariance_scaled%*%diag(scales)
beta <- theta[seq_len(p)]
gamma_ml <- theta[p+seq_len(q)]
log_sigma <- theta[p+q+1L]
athrho <- theta[p+q+2L]
sigma_ml <- exp(log_sigma); rho_ml <- tanh(athrho); lambda_ml <- rho_ml*sigma_ml

# Same-sample rho=0 likelihood: separate probit and selected Gaussian OLS.
sigma_null <- sqrt(ols_sse/n1)
independent_ll <- probit$ll+sum(dnorm(ols$residuals/sigma_null,log=TRUE)-log(sigma_null))
lr <- 2*(ml_fit$ll-independent_ll)

quantities <- list()
emit <- function(quantity,value,i=0,j=0) {
  quantities[[length(quantities)+1L]] <<- data.frame(quantity=quantity,i=i,j=j,value=as.numeric(value))
}
for (i in seq_len(q)) {
  emit("probit_value",gamma[i],i)
  emit("probit_standard_error",sqrt(probit_covariance[i,i]),i)
  for (j in seq_len(q)) emit("probit_covariance",probit_covariance[i,j],i,j)
}
for (i in seq_len(p)) {
  emit("twopart_ols_value",ols$coefficients[i],i)
  emit("twopart_ols_standard_error",sqrt(ols_covariance[i,i]),i)
  emit("heckman_ml_outcome_value",beta[i],i)
  emit("heckman_ml_outcome_standard_error",sqrt(ml_covariance[i,i]),i)
  for (j in seq_len(p)) emit("twopart_ols_covariance",ols_covariance[i,j],i,j)
}
for (i in seq_len(q)) {
  emit("heckman_ml_selection_value",gamma_ml[i],i)
  emit("heckman_ml_selection_standard_error",sqrt(ml_covariance[p+i,p+i]),i)
}
for (i in seq_len(p+q+2L)) for (j in seq_len(p+q+2L)) emit("heckman_ml_covariance",ml_covariance[i,j],i,j)
for (i in seq_len(p+1L)) {
  emit("heckman_twostep_outcome_value",two_step_beta[i],i)
  emit("heckman_twostep_outcome_standard_error",sqrt(two_step_covariance[i,i]),i)
  for (j in seq_len(p+1L)) emit("heckman_twostep_outcome_covariance",two_step_covariance[i,j],i,j)
}
for (i in seq_len(q)) {
  emit("heckman_twostep_selection_value",gamma[i],i)
  emit("heckman_twostep_selection_standard_error",sqrt(probit_covariance[i,i]),i)
}
summary_values <- c(
  n=n,selected_count=n1,unselected_count=n-n1,probit_loglikelihood=probit$ll,
  twopart_ols_loglikelihood=ols_loglik,twopart_loglikelihood=probit$ll+ols_loglik,
  heckman_ml_loglikelihood=ml_fit$ll,heckman_ml_independent_loglikelihood=independent_ll,
  heckman_ml_independence_lr=lr,heckman_ml_sigma=sigma_ml,heckman_ml_rho=rho_ml,
  heckman_ml_lambda=lambda_ml,heckman_ml_lnsigma=log_sigma,
  heckman_ml_athrho=athrho,heckman_twostep_sigma=sigma_two_step,
  heckman_twostep_rho=rho_two_step,heckman_twostep_lambda=alpha,
  heckman_ml_score=max(abs(ml_fit$score*scales))/n)
for (name in names(summary_values)) emit(name,summary_values[[name]])
write.csv(do.call(rbind,quantities),args[2],row.names=FALSE,quote=TRUE)

ml_index <- drop(Z%*%gamma_ml)
ml_probability <- pnorm(ml_index)
ml_mills <- inverse_mills(ml_index)
ml_xb <- drop(X_all%*%beta)
two_step_xb <- drop(X_all%*%two_step_beta[seq_len(p)])
two_step_conditional <- two_step_xb+alpha*mills
ml_conditional <- ml_xb+lambda_ml*ml_mills
predictions <- data.frame(source_row=seq_len(n),DMED=dat$DMED,MED=dat$MED,LNMED=dat$LNMED,
  probit_probability=selection_probability,twopart_lnmed_xb=drop(X_all%*%ols$coefficients),
  ml_xb=ml_xb,ml_probability=ml_probability,ml_mills=ml_mills,ml_ycond=ml_conditional,
  twostep_xb=two_step_xb,twostep_probability=selection_probability,
  twostep_mills=mills,twostep_ycond=two_step_conditional)
write.csv(predictions,args[3],row.names=FALSE,na="")
cat("Reference:",R.version.string,"Base R glm/lm, BFGS, and independently finite-differenced observed information\n",file=stderr())
