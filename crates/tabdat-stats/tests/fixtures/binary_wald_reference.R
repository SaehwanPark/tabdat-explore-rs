# Development-only base R 4.6.2 reference for binary linear Wald restrictions.
# Rscript <this file> <fixture directory>
# No RNG or contributed package is needed.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
out <- if(length(args)) args[[1]] else tempdir()
dir.create(out, recursive=TRUE, showWarnings=FALSE)
i <- 1:40
x1 <- ((i*7)%%17)/5-1.6
x2 <- ((i*11)%%19)/7-9/7
y <- as.integer(((i*13)%%37)<18+4*x1-3*x2)
group <- (i*7)%%9
d <- data.frame(y,x1,x2,group)
write.csv(d,file.path(out,'binary_wald_data.csv'),row.names=FALSE,na='')

rows <- list()
reference <- function(family, covariance) {
  fit <- glm(y~x1+x2,data=d,family=binomial(link=family),
    control=glm.control(epsilon=1e-13,maxit=100))
  stopifnot(fit$converged,!fit$boundary)
  X <- model.matrix(fit)
  b <- coef(fit)
  parts <- function(b) {
    eta <- drop(X%*%b)
    if(family=='logit') {
      p <- plogis(eta); score <- y-p; curvature <- p*(1-p)
    } else {
      p <- pnorm(eta); pdf <- dnorm(eta); a <- pdf/(p*(1-p))
      score <- (y-p)*a
      curvature <- pdf*a+score*(eta+(1-2*p)*a)
    }
    list(score=score,information=crossprod(X,X*curvature))
  }
  for(iteration in 1:20) {
    current <- parts(b)
    gradient <- drop(crossprod(X,current$score))
    if(max(abs(gradient))<1e-12) break
    b <- b+solve(current$information,gradient)
  }
  current <- parts(b)
  stopifnot(max(abs(crossprod(X,current$score)))<1e-12)
  bread <- solve(current$information)
  V <- if(covariance=='model') {
    bread
  } else {
    scores <- X*current$score
    units <- if(covariance=='cluster') {
      scores <- rowsum(scores,d$group,reorder=FALSE)
      length(unique(d$group))
    } else nrow(X)
    units/(units-1)*bread%*%crossprod(scores)%*%bread
  }
  restrictions <- list(
    one=list(R=matrix(c(0,1,1),nrow=1),r=0.2),
    joint=list(R=rbind(c(0,1,1),c(0,1,-1)),r=c(0.2,0.4))
  )
  for(name in names(restrictions)) {
    restriction <- restrictions[[name]]
    difference <- drop(restriction$R%*%b)-restriction$r
    tested_covariance <- restriction$R%*%V%*%t(restriction$R)
    statistic <- drop(t(difference)%*%solve(tested_covariance,difference))
    df <- nrow(restriction$R)
    rows[[length(rows)+1L]] <<- data.frame(
      family,covariance,restriction=name,df,statistic,
      p_value=pchisq(statistic,df,lower.tail=FALSE)
    )
  }
}
for(family in c('logit','probit'))
  for(covariance in c('model','robust','cluster'))
    reference(family,covariance)
write.csv(do.call(rbind,rows),file.path(out,'binary_wald_reference.csv'),
  row.names=FALSE,quote=FALSE)
cat(R.version.string,'\n')
