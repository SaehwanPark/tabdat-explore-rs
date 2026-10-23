# Development-only R4.6.1/survival3.8.6 independent conditional-choice reference.
# Rscript reference_clogit.R Nldata.csv R_clogit_predictions.csv > R_clogit_quantities.csv
options(digits=17)
library(survival)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L)
wide <- read.csv(args[1])
stopifnot(nrow(wide)==1182L,identical(wide$source_row,seq_len(1182L)),all(is.finite(as.matrix(wide))))
labels <- c("beach","pier","private","charter")
d <- do.call(rbind,lapply(1:4,function(j) data.frame(id=wide$source_row,alternative=j,
  chosen=wide[[paste0("d",labels[j])]],p=wide[[paste0("p",labels[j])]],q=wide[[paste0("q",labels[j])]])))
d <- d[order(d$id,d$alternative),];rownames(d)<-NULL
stopifnot(all(d$chosen==as.numeric(rep(wide$mode,each=4)==d$alternative)),all(rowsum(d$chosen,d$id)==1))
fit <- survival::clogit(chosen~p+q+strata(id),data=d,method="exact",
  control=survival::coxph.control(eps=1e-12,toler.chol=1e-14,iter.max=100))
stopifnot(!anyNA(coef(fit)),all(is.finite(vcov(fit))))
b <- coef(fit);V <- vcov(fit);X <- as.matrix(d[c("p","q")]);G <- length(unique(d$id))
probability <- function(X) {
  weights <- exp(drop(X%*%b))
  weights/rep(drop(rowsum(weights,d$id)),each=4)
}
pr <- probability(X)
score <- colSums(X*(d$chosen-pr));stopifnot(max(abs(score))<1e-7)
# Cox expected counts equal single-event conditional choice probabilities on
# the original risk sets; out-of-sample scenarios must be renormalized anew.
stopifnot(max(abs(pr-predict(fit,type="expected")))<1e-12)
ll <- as.numeric(logLik(fit));null <- -G*log(4);lr <- 2*(ll-null)
stopifnot(abs(ll-fit$loglik[2])<1e-10,abs(null-fit$loglik[1])<1e-10)
rows <- list()
emit <- function(quantity,i,j,value) {
  rows[[length(rows)+1L]] <<- data.frame(quantity=quantity,i=i,j=j,value=value)
}
for(name in c("observation_count","choice_set_count","choice_set_size","log_likelihood",
  "null_log_likelihood","likelihood_ratio","likelihood_ratio_df","likelihood_ratio_p_value","mcfadden_r_squared")) {
  value <- switch(name,observation_count=nrow(d),choice_set_count=G,choice_set_size=4,
    log_likelihood=ll,null_log_likelihood=null,likelihood_ratio=lr,likelihood_ratio_df=2,
    likelihood_ratio_p_value=pchisq(lr,2,lower.tail=FALSE),mcfadden_r_squared=1-ll/null)
  emit(name,0,0,value)
}
for(i in 1:2) {
  se <- sqrt(V[i,i]);z <- b[i]/se
  for(name in c("value","standard_error","statistic","p_value","confidence_lower","confidence_upper")) {
    value <- switch(name,value=b[i],standard_error=se,statistic=z,p_value=2*pnorm(-abs(z)),
      confidence_lower=b[i]-qnorm(.975)*se,confidence_upper=b[i]+qnorm(.975)*se)
    emit(paste0("coefficient_",name),i-1,0,value)
  }
  for(j in 1:2) emit("covariance",i-1,j-1,V[i,j])
}
expected <- d
expected$pbase<-d$p;expected$qbase<-d$q;expected$pr<-pr
for(r in 1:2) for(alternative in 1:4) {
  changed <- X;step <- c(1,.001)[r]
  changed[d$alternative==alternative,r] <- changed[d$alternative==alternative,r]+step
  next_pr <- probability(changed)
  name <- paste0(c("p_price","p_catch")[r],alternative)
  expected[[name]] <- next_pr
  # Price entries are percentage-point changes for +$1; catch entries are
  # probability changes divided by .001, exactly as the author's section2B.
  contrasts <- c(100,1000)[r]*(next_pr-pr)
  for(outcome in 1:4) emit(paste0(c("price_contrast","catch_contrast")[r]),outcome-1,alternative-1,
    mean(contrasts[d$alternative==outcome]))
}
for(alternative in 1:4) emit("mean_probability",alternative-1,0,mean(pr[d$alternative==alternative]))
write.csv(expected,args[2],row.names=FALSE)
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE)
