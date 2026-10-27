# Base-R score fit and observed within-bin group contrasts.
args <- commandArgs(trailingOnly=TRUE)
if(length(args)!=2) stop('usage: Rscript stratified_reference.R nswpsid.da1 output-directory')
if(dir.exists(args[2])) stop('output directory exists')
x <- read.table(args[1],col.names=c('treat','age','educ','black','hisp','marr','re74','re75','re78','u74','u75'))
stopifnot(nrow(x)==2675,sum(x$treat)==185,all(x$treat %in% c(0,1)))
x <- transform(x,age_s=age/10,age_s2=(age/10)^2,educ_s=educ/10,educ_s2=(educ/10)^2,
 nodegree=as.numeric(educ<12),re74_s=re74/10000,re75_s=re75/10000,
 re74_s2=(re74/10000)^2,re75_s2=(re75/10000)^2,u74black=as.numeric(re74==0)*black)
fit <- glm(treat~age_s+age_s2+educ_s+educ_s2+marr+nodegree+black+hisp+re74_s+re75_s+re74_s2+re75_s2+u74black,
 family=binomial(),data=x,control=glm.control(epsilon=1e-13,maxit=100))
stopifnot(fit$converged,fit$rank==ncol(model.matrix(fit)))
x$pscore <- predict(fit,type='response')
lower <- max(min(x$pscore[x$treat==0]),min(x$pscore[x$treat==1]))
upper <- min(max(x$pscore[x$treat==0]),max(x$pscore[x$treat==1]))
stopifnot(lower<upper)
x$in_overlap <- x$pscore>=lower & x$pscore<=upper
x$stratum <- ifelse(x$in_overlap,ceiling(10*x$pscore),NA_integer_)
cells <- do.call(rbind,lapply(1:10,function(i) {
  z <- x[x$in_overlap & x$stratum==i,];t <- z$re78[z$treat==1];c <- z$re78[z$treat==0]
  nt <- length(t);nc <- length(c);mt <- if(nt) mean(t) else NA_real_;mc <- if(nc) mean(c) else NA_real_
  status <- if(nt==0 && nc==0) 'empty' else if(nt==0) 'no_treated' else if(nc==0) 'no_control' else 'supported'
  if(status=='supported') {
    # A separately fitted regression checks the difference-of-means identity.
    stopifnot(abs(unname(coef(lm(re78~treat,data=z))['treat'])-(mt-mc))<1e-7)
  }
  data.frame(stratum=i,treated_n=nt,control_n=nc,treated_mean=mt,control_mean=mc,
    difference=if(status=='supported') mt-mc else NA_real_,status=status)
}))
# Compare the nominal covariate-adjusted regression with printed unadjusted results.
adjusted <- do.call(rbind,lapply(c(1,5),function(i) {
  z <- x[x$in_overlap & x$stratum==i,]
  model <- lm(re78~treat+age+I(age^2)+educ+nodegree+black+hisp+re74+re75,data=z)
  estimate <- unname(coef(model)['treat'])
  stopifnot(is.finite(estimate),abs(estimate-cells$difference[i])>100)
  data.frame(stratum=i,adjusted_treatment=estimate)
}))
valid <- cells$status=='supported'
summary <- data.frame(common_range_treated=sum(cells$treated_n),supported_treated=sum(cells$treated_n[valid]),
 unsupported_treated=sum(cells$treated_n[!valid]),supported_strata=sum(valid),
 treated_weighted_difference=sum(cells$treated_n[valid]*cells$difference[valid])/sum(cells$treated_n[valid]))
dir.create(args[2])
write.csv(cells,file.path(args[2],'cells.csv'),row.names=FALSE,na='')
write.csv(adjusted,file.path(args[2],'adjusted.csv'),row.names=FALSE)
write.csv(summary,file.path(args[2],'summary.csv'),row.names=FALSE)
write.csv(data.frame(source_row=seq_len(nrow(x)),treat=x$treat,re78=x$re78,pscore=x$pscore,
 lower_bound=lower,upper_bound=upper,in_overlap=x$in_overlap,stratum=x$stratum),file.path(args[2],'rows.csv'),row.names=FALSE,na='')
