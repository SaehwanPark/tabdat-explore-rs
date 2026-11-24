# Derive Table20.5 normal-reference combinations from independently checked fits.
# Rscript lincom_reference.R R_poisson.csv R_nb2.csv > R_lincom.csv
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
  # In this source design, beta[1:2] is the ordered LC, IDP pair.
  beta <- vapply(0:17,function(i)
    value(fit_reference,robust,'coefficient_value',i),numeric(1))
  V <- outer(0:17,0:17,Vectorize(function(i,j)
    value(fit_reference,robust,'covariance',i,j)))
  combinations <- list(
    lc=list(weights=replace(rep(0,18),2,1),offset=0),
    contrast=list(weights=replace(replace(rep(0,18),2,1),3,-1),offset=2),
    sum=list(weights=replace(replace(rep(0,18),2,1),3,1),offset=-0.25)
  )
  for (name in names(combinations)) {
    combination <- combinations[[name]]
    estimate <- combination$offset+sum(combination$weights*beta)
    standard_error <- sqrt(drop(t(combination$weights)%*%V%*%combination$weights))
    statistic <- estimate/standard_error
    rows[[length(rows)+1L]] <<- data.frame(
      family,covariance=if(robust)'robust' else 'model',combination=name,
      estimate,standard_error,statistic,p_value=2*pnorm(-abs(statistic)),
      ci_lower=estimate-qnorm(0.975)*standard_error,
      ci_upper=estimate+qnorm(0.975)*standard_error,ci_level=95
    )
  }
}
for (robust in c(FALSE,TRUE)) {
  reference('poisson',poisson,robust)
  reference('nb2',nb2,robust)
}
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE)
