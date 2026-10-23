# Development-only R4.6.1/survival3.8.6 exact conditional-choice reference.
# Rscript reference_combined.R Nldata.csv R_combined_predictions.csv > R_combined_quantities.csv
options(digits=17)
library(survival)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L)
w <- read.csv(args[1])
stopifnot(nrow(w)==1182L,identical(w$source_row,seq_len(1182L)),all(is.finite(as.matrix(w))))
labels <- c("beach","pier","private","charter")
d <- do.call(rbind,lapply(1:4,function(j) data.frame(id=w$source_row,alternative=j,
  chosen=w[[paste0("d",labels[j])]],p=w[[paste0("p",labels[j])]],q=w[[paste0("q",labels[j])]],
  ydiv1000=w$income/1000)))
d <- d[order(d$id,d$alternative),];rownames(d)<-NULL
stopifnot(all(d$chosen==as.numeric(rep(w$mode,each=4)==d$alternative)),all(rowsum(d$chosen,d$id)==1))
for(j in 2:4) {
  d[[paste0("dummy",j)]] <- as.numeric(d$alternative==j)
  d[[paste0("d",j,"y")]] <- d[[paste0("dummy",j)]]*d$ydiv1000
}
encoded_names <- c("dummy2","dummy3","dummy4","d2y","d3y","d4y")
combined_names <- c("p","q",encoded_names)
control <- survival::coxph.control(eps=1e-12,toler.chol=1e-14,iter.max=100)
encoded <- survival::clogit(chosen~dummy2+dummy3+dummy4+d2y+d3y+d4y+strata(id),data=d,method="exact",control=control)
combined <- survival::clogit(chosen~p+q+dummy2+dummy3+dummy4+d2y+d3y+d4y+strata(id),data=d,method="exact",control=control)
probability <- function(data,fit,names) {
  eta <- drop(as.matrix(data[names])%*%coef(fit))
  centers <- rep(apply(matrix(eta,nrow=4),2,max),each=4)
  weights <- exp(eta-centers)
  weights/rep(drop(rowsum(weights,data$id)),each=4)
}
quantities <- list()
emit <- function(model,quantity,i,j,value) {
  quantities[[length(quantities)+1L]] <<- data.frame(model=model,quantity=quantity,i=i,j=j,value=value)
}
for(model in c("encoded","combined")) {
  fit <- get(model);names <- get(paste0(model,"_names"));X <- as.matrix(d[names]);b <- coef(fit);V <- vcov(fit)
  stopifnot(!anyNA(b),all(is.finite(V)),identical(names(b),names))
  pr <- probability(d,fit,names)
  stopifnot(max(abs(colSums(X*(d$chosen-pr))))<1e-7,max(abs(pr-predict(fit,type="expected")))<1e-12)
  ll <- as.numeric(logLik(fit));null <- -1182*log(4);lr <- 2*(ll-null)
  stopifnot(abs(null-fit$loglik[1])<1e-10)
  for(name in c("observation_count","choice_set_count","choice_set_size","log_likelihood","null_log_likelihood",
    "likelihood_ratio","likelihood_ratio_df","likelihood_ratio_p_value","mcfadden_r_squared")) {
    value <- switch(name,observation_count=nrow(d),choice_set_count=1182,choice_set_size=4,log_likelihood=ll,
      null_log_likelihood=null,likelihood_ratio=lr,likelihood_ratio_df=length(b),
      likelihood_ratio_p_value=pchisq(lr,length(b),lower.tail=FALSE),mcfadden_r_squared=1-ll/null)
    emit(model,name,0,0,value)
  }
  for(i in seq_along(b)) {
    se <- sqrt(V[i,i]);z <- b[i]/se
    for(name in c("value","standard_error","statistic","p_value","confidence_lower","confidence_upper")) {
      value <- switch(name,value=b[i],standard_error=se,statistic=z,p_value=2*pnorm(-abs(z)),
        confidence_lower=b[i]-qnorm(.975)*se,confidence_upper=b[i]+qnorm(.975)*se)
      emit(model,paste0("coefficient_",name),i-1,0,value)
    }
    for(j in seq_along(b)) emit(model,"covariance",i-1,j-1,V[i,j])
  }
}
expected <- d[c("id","alternative","chosen","p","q","ydiv1000")]
expected$mnl_probability <- probability(d,encoded,encoded_names)
for(name in encoded_names) expected[[name]] <- d[[name]]
expected$ybase<-d$ydiv1000;expected$pbase<-d$p;expected$qbase<-d$q
expected$encoded_mnl <- expected$mnl_probability
expected$baseline <- probability(d,combined,combined_names)
for(scenario in c("income","price","catch")) {
  changed <- d
  if(scenario=="income") {
    changed$ydiv1000 <- changed$ydiv1000+1
    for(j in 2:4) changed[[paste0("d",j,"y")]] <- changed[[paste0("dummy",j)]]*changed$ydiv1000
  } else if(scenario=="price") changed$p[d$alternative==4] <- changed$p[d$alternative==4]+1
  else changed$q[d$alternative==4] <- changed$q[d$alternative==4]+.001
  expected[[scenario]] <- probability(changed,combined,combined_names)
  factor <- switch(scenario,income=1,price=100,catch=1000)
  for(j in 1:4) emit("combined",paste0(scenario,"_contrast"),j-1,0,
    mean(factor*(expected[[scenario]]-expected$baseline)[d$alternative==j]))
}
for(j in 1:4) emit("combined","mean_probability",j-1,0,mean(expected$baseline[d$alternative==j]))
# Income division can require all17 digits for an exact parsed-f64 source bind;
# write.csv's numeric encoding otherwise rounds independently of options(digits).
expected[] <- lapply(expected,function(x) sprintf("%.17g",x))
write.csv(expected,args[2],row.names=FALSE)
write.csv(do.call(rbind,quantities),stdout(),row.names=FALSE,quote=FALSE)
