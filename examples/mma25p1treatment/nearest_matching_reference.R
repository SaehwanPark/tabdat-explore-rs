# Independent base-R score fit and exact-tie nearest-control comparisons.
args <- commandArgs(trailingOnly=TRUE)
if(length(args)!=2) stop('usage: Rscript nearest_matching_reference.R nswpsid.da1 output-directory')
if(dir.exists(args[2])) stop('output directory exists')
x <- read.table(args[1],col.names=c('treat','age','educ','black','hisp','marr','re74','re75','re78','u74','u75'))
stopifnot(nrow(x)==2675,sum(x$treat)==185,all(x$treat %in% c(0,1)))
x <- transform(x,age_s=age/10,age_s2=(age/10)^2,educ_s=educ/10,educ_s2=(educ/10)^2,
 nodegree=as.numeric(educ<12),re74_s=re74/10000,re75_s=re75/10000,
 re74_s2=(re74/10000)^2,re75_s2=(re75/10000)^2,u74black=as.numeric(re74==0)*black)
fit <- glm(treat~age_s+age_s2+educ_s+educ_s2+marr+nodegree+black+hisp+re74_s+re75_s+re74_s2+re75_s2+u74black,
 family=binomial(),data=x,control=glm.control(epsilon=1e-13,maxit=100))
stopifnot(fit$converged,fit$rank==ncol(model.matrix(fit)))
p <- unname(predict(fit,type='response'));t <- x$treat;y <- x$re78
lower <- max(min(p[t==0]),min(p[t==1]));upper <- min(max(p[t==0]),max(p[t==1]))
keep <- p>=lower & p<=upper; it <- which(keep & t==1);ic <- which(keep & t==0)
stopifnot(length(it)>=2,length(ic)>=2,lower<upper)
pairs <- do.call(rbind,lapply(it,function(i){d <- abs(p[i]-p[ic]);nearest <- min(d); ids <- ic[d==nearest]
  data.frame(treated_id=i,treated_score=p[i],treated_outcome=y[i],control_id=ids,control_score=p[ids],
    control_outcome=y[ids],distance=nearest,nearest_distance=nearest,tied_controls=length(ids),matched_outcome=mean(y[ids]))}))
treated <- pairs[!duplicated(pairs$treated_id),]
reuse <- tapply(1/pairs$tied_controls,pairs$control_id,sum)
summary <- data.frame(matched_treated=nrow(treated),distinct_controls=length(reuse),
 contrast=mean(treated$treated_outcome-treated$matched_outcome),max_distance=max(treated$distance),
 tied_treated=sum(treated$tied_controls>1),max_reuse=max(reuse))
dir.create(args[2])
write.csv(pairs,file.path(args[2],'pairs.csv'),row.names=FALSE)
write.csv(summary,file.path(args[2],'summary.csv'),row.names=FALSE)
