# Development-only R 4.6.1 reference. No production R dependency.
# Rscript .../binary_robust_reference.R > .../binary_robust_reference.csv
# Optional Nldata.csv argument emits the exact MMA 14.2 fishing references.
# --cluster emits synthetic G=9 references; --cluster Contraception.csv [R.csv]
# emits the specified external district-clustered reference and predictions.
# ML sandwich correction N/(N-1): Stata User's Guide 20.22.2 (not OLS HC1).
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
clustered <- length(args)>0L && args[[1]]=="--cluster"
if(clustered) args <- args[-1]
stopifnot(length(args) <= if(clustered) 2L else 1L)
rows <- list()
emit <- function(family, intercept, two, quantity, i, j, value) {
  rows[[length(rows)+1L]] <<- data.frame(family=family, intercept=intercept,
    two=two, quantity=quantity, i=i-1L, j=j-1L, value=value)
}
reference <- function(d, family, intercept, two, groups=NULL) {
  formula <- if(two) y~x1+x2 else y~x1
  if(!intercept) formula <- update(formula,~.-1)
  fit <- glm(formula, data=d, family=binomial(link=family),
    control=glm.control(epsilon=1e-13,maxit=100))
  stopifnot(fit$converged, !fit$boundary)
  X <- model.matrix(fit); y <- d$y; b <- coef(fit)
  parts <- function(b) {
    z <- drop(X%*%b)
    if(family=="logit") {
      p <- plogis(z); score <- y-p; curvature <- p*(1-p)
      derivative <- curvature; second <- curvature*(1-2*p)
    } else {
      p <- pnorm(z); pdf <- dnorm(z); a <- pdf/(p*(1-p))
      score <- (y-p)*a
      curvature <- pdf*a + score*(z+(1-2*p)*a)
      derivative <- pdf; second <- -z*pdf
    }
    list(p=p, score=score, information=crossprod(X,X*curvature),
      derivative=derivative, second=second)
  }
  # glm's deviance-change stop is not necessarily a score fixed point for
  # probit. Polish on observed information, as in probit_reference.R.
  for(it in 1:20) {
    c <- parts(b); gradient <- drop(crossprod(X,c$score))
    if(max(abs(gradient)) < 1e-12) break
    b <- b + solve(c$information,gradient)
  }
  c <- parts(b)
  stopifnot(max(abs(crossprod(X,c$score))) < 1e-12)
  bread <- solve(c$information)
  scores <- X*c$score
  if(!is.null(groups)) scores <- rowsum(scores,groups,reorder=FALSE)
  units <- nrow(scores)
  stopifnot(units>1L)
  V <- units/(units-1) * bread%*%crossprod(scores)%*%bread
  se <- sqrt(diag(V)); critical <- qnorm(.975)
  for(i in seq_along(b)) {
    for(q in c("coefficient","se","z","p","lower","upper")) {
      v <- switch(q, coefficient=b[i], se=se[i], z=b[i]/se[i],
        p=2*pnorm(-abs(b[i]/se[i])), lower=b[i]-critical*se[i], upper=b[i]+critical*se[i])
      emit(family,intercept,two,q,i,1,v)
    }
    for(j in seq_along(b)) emit(family,intercept,two,"covariance",i,j,V[i,j])
  }
  slopes <- seq_along(b)[seq_along(b)>as.integer(intercept)]
  for(j in slopes) {
    ame <- b[j]*mean(c$derivative)
    g <- b[j]*colMeans(X*c$second); g[j] <- g[j]+mean(c$derivative)
    ame_se <- sqrt(drop(t(g)%*%V%*%g))
    for(q in c("ame","ame_se","ame_z","ame_p","ame_lower","ame_upper")) {
      v <- switch(q, ame=ame, ame_se=ame_se, ame_z=ame/ame_se,
        ame_p=2*pnorm(-abs(ame/ame_se)), ame_lower=ame-critical*ame_se, ame_upper=ame+critical*ame_se)
      emit(family,intercept,two,q,j-as.integer(intercept),1,v)
    }
  }
  invisible(list(probability=c$p,index=drop(X%*%b)))
}
if(clustered && length(args)>0L) {
  d <- read.csv(args[[1]])
  stopifnot(nrow(d)==1934L, identical(d$woman,seq_len(1934L)),
    length(unique(d$district))==60L, all(d$use %in% c("N","Y")),
    all(complete.cases(d[c("district","age")])))
  d$y <- as.integer(d$use=="Y"); d$x1 <- d$age
  stopifnot(sum(d$y)==759L)
  predictions <- data.frame(woman=d$woman,district=d$district,y=d$y,age=d$age)
  for(family in c("logit","probit")) {
    r <- reference(d,family,TRUE,FALSE,d$district)
    predictions[[paste0(family,"_pr")]] <- r$probability
    predictions[[paste0(family,"_xb")]] <- r$index
  }
  if(length(args)==2L) write.csv(predictions,args[[2]],row.names=FALSE)
} else if(length(args)==1L) {
  d <- read.csv(args[[1]])
  stopifnot(nrow(d)==1182L, identical(d$source_row,seq_len(1182L)))
  d <- d[d$mode %in% c(2,4), ]; d <- d[order(d$source_row), ]
  stopifnot(nrow(d)==630L, sum(d$mode==4)==452L)
  d$y <- as.integer(d$mode==4); d$x1 <- log(d$pcharter/d$ppier)
  for(family in c("logit","probit")) reference(d,family,TRUE,FALSE)
} else {
  i <- 1:40
  x1 <- ((i*7)%%17)/5-1.6; x2 <- ((i*11)%%19)/7-9/7
  d <- data.frame(x1=x1,x2=x2,y=as.integer(((i*13)%%37)<18+4*x1-3*x2))
  for(family in c("logit","probit")) for(intercept in c(TRUE,FALSE)) for(two in c(TRUE,FALSE)) {
    reference(d,family,intercept,two,if(clustered) (i*7)%%9 else NULL)
  }
}
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE)
