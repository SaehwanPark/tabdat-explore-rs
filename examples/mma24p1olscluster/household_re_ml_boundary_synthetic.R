# Construct a boundary control from the inspectable unbalanced household fixture.
# Rscript household_re_ml_boundary_synthetic.R interior-input.csv fresh-output.csv
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=2 || file.exists(args[2])) stop('requires input and fresh output CSV')
d <- read.csv(args[1]);keep <- !is.na(d$lnexp12m);x <- d[keep,]
X <- model.matrix(~lnhhexp+age+sex+hhsize+farm+educ,x)
Xm <- apply(X,2,function(v)ave(v,x$commune));Xw <- X-Xm
# Zero commune means and residual orthogonality put the ML optimum at u=0.
v <- sin(x$source_row*1.31)+cos(x$source_row*.45);v <- v-ave(v,x$commune)
e <- lm.fit(Xw[,-1],v)$residuals
stopifnot(sum(e^2)>1,max(abs(ave(e,x$commune)))<1e-10)
x$lnexp12m <- drop(X%*%c(.2,.6,.01,.1,.03,.2,-.04))+e
d$lnexp12m[keep] <- x$lnexp12m
# Bounded decimal precision keeps synthetic input identically parsed by R/DuckDB.
d[] <- lapply(d,function(v)ifelse(is.na(v),NA_character_,sprintf('%.12g',v)))
write.csv(d,args[2],row.names=FALSE,quote=FALSE,na='')
