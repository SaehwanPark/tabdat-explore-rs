# Derive Table20.5 test references from the independently validated fit outputs.
# Rscript wald_reference.R R_reference.csv R_nb2_reference.csv > R_wald_reference.csv
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L)
poisson <- read.csv(args[[1]])
nb2 <- read.csv(args[[2]])
rows <- list()
value <- function(reference,robust,quantity,i,j=NULL) {
  keep <- reference$case=='mma20p1count' & reference$robust==as.integer(robust) &
    reference$quantity==quantity & reference$i==i
  if (!is.null(j)) keep <- keep & reference$j==j
  selected <- reference$value[keep]
  stopifnot(length(selected)==1L)
  selected
}
reference <- function(family,fit_reference,robust) {
  beta <- vapply(0:17,function(i)
    value(fit_reference,robust,'coefficient_value',i),numeric(1))
  V <- outer(0:17,0:17,Vectorize(function(i,j)
    value(fit_reference,robust,'covariance',i,j)))
  restrictions <- list(
    one=list(R=matrix(c(rep(0,1),1,rep(0,16)),nrow=1L),r=0),
    joint=list(R=rbind(c(0,1,1,rep(0,15)),c(0,0,1,rep(0,15))),r=c(0,0))
  )
  for (name in names(restrictions)) {
    restriction <- restrictions[[name]]
    difference <- drop(restriction$R%*%beta)-restriction$r
    tested_covariance <- restriction$R%*%V%*%t(restriction$R)
    statistic <- drop(t(difference)%*%solve(tested_covariance,difference))
    df <- nrow(restriction$R)
    rows[[length(rows)+1L]] <<- data.frame(
      family,covariance=if(robust)'robust' else 'model',restriction=name,df,
      statistic,p_value=pchisq(statistic,df,lower.tail=FALSE)
    )
  }
}
for (robust in c(FALSE,TRUE)) {
  reference('poisson',poisson,robust)
  reference('nb2',nb2,robust)
}
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE)
