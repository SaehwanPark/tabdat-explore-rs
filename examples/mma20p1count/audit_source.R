# Optional development-only DTA/ASCII sample audit; foreign is not a runtime dependency.
# Rscript audit_source.R mma20p1count.asc randdata.dta
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L)
values <- scan(args[1],quiet=TRUE)
stopifnot(length(values)==20186L*18L)
d <- as.data.frame(matrix(values,ncol=18,byrow=TRUE))
names(d) <- c("MDU","LC","IDP","LPI","FMDE","PHYSLIM","NDISEASE","HLTHG",
  "HLTHF","HLTHP","LINC","LFAM","EDUCDEC","AGE","FEMALE","CHILD","FEMCHILD","BLACK")
r <- foreign::read.dta(args[2],convert.factors=FALSE)
stopifnot(nrow(r)==20190L,identical(sort(unique(r$year)),as.numeric(1:5)),
  identical(which(is.na(r$educdec)),c(9746L,10153L,10400L,10859L)),anyDuplicated(r$zper)>0L)
map <- c("mdvis","logc","idp","lpi","fmde","physlm","disea","hlthg","hlthf",
  "hlthp","linc","lfam","educdec","xage","female","child","fchild","black")
original <- r[!is.na(r$educdec),map]
names(original) <- names(d)
stopifnot(nrow(original)==nrow(d),!anyNA(original),sum(d$MDU)==57746,
  sum(d$MDU==0)==6308,max(d$MDU)==77,identical(d$MDU,as.numeric(original$MDU)))
differences <- vapply(seq_along(d),function(j)max(abs(d[[j]]-original[[j]])),numeric(1))
stopifnot(all(differences<5e-6))
print(setNames(differences,names(d)))
fit <- function(data) glm(MDU~.,data=data,family=poisson(),control=glm.control(epsilon=1e-13,maxit=100))
a <- fit(d); b <- fit(original)
cat("All five years; 20190 original rows; four education-missing rows excluded.\n")
cat("Every count and retained export position matches; predictors differ <5e-6 from ASCII quantization, NOT an estimator tolerance.\n")
cat("ASCII logLik",as.numeric(logLik(a)),"original DTA logLik",as.numeric(logLik(b)),
  "max coefficient difference",max(abs(coef(a)-coef(b))),"\n")
cat("prepared_row is not zper, a year, or an original DTA row ID. Original IDs are not in the ASCII.\n")
