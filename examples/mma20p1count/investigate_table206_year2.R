# Independent investigation of the year-2-only hypothesis for Table 20.6.
# Development-only dependencies: foreign and MASS; neither is used at runtime.
# Run after downloading and checksum-verifying randdata.dta as documented in README.md.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==1L)

source_data <- foreign::read.dta(args[[1]],convert.factors=FALSE)
stopifnot(nrow(source_data)==20190L,
  identical(sort(unique(source_data$year)),as.numeric(1:5)),
  identical(which(is.na(source_data$educdec)),c(9746L,10153L,10400L,10859L)))

predictors <- c("logc","idp","lpi","fmde","physlm","disea","hlthg","hlthf",
  "hlthp","linc","lfam","educdec","xage","female","child","fchild","black")
year2 <- source_data[source_data$year==2 & !is.na(source_data$educdec),c("mdvis",predictors)]
names(year2) <- c("MDU","LC","IDP","LPI","FMDE","PHYSLIM","NDISEASE","HLTHG",
  "HLTHF","HLTHP","LINC","LFAM","EDUCDEC","AGE","FEMALE","CHILD","FEMCHILD","BLACK")
stopifnot(nrow(year2)==5574L,!anyNA(year2))

fit <- MASS::glm.nb(MDU~.,data=year2,
  control=stats::glm.control(epsilon=1e-10,maxit=500))
stopifnot(isTRUE(fit$converged),is.finite(fit$theta),fit$theta>0)

# NB2 parameterization: Var(Y|X)=mu+mu^2/theta.
mu <- stats::fitted(fit)
theta <- fit$theta
p0 <- (theta/(theta+mu))^theta
p1 <- p0*theta*mu/(theta+mu)
p2 <- p1*(theta+1)*mu/(2*(theta+mu))
shares <- 100*c(mean(p0),mean(p1),mean(p2))
stopifnot(all(is.finite(shares)),all(shares>=0),all(shares<=100))

cat("R",R.version.string,"MASS",as.character(utils::packageVersion("MASS")),"\n")
cat("Year-2 complete sample N",nrow(year2),"converged",fit$converged,
  "theta",theta,"alpha",1/theta,"logLik",as.numeric(stats::logLik(fit)),"\n")
cat("Mean fitted NB2 probabilities for counts 0, 1, and 2 (%):",
  paste(sprintf("%.12f",shares),collapse=", "),"\n")
