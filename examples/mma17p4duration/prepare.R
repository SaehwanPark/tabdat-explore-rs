# Development-only conversion of the exact original DTA; application uses CSV.
# Rscript prepare.R ema1996.dta output.csv
args <- commandArgs(TRUE)
stopifnot(length(args)==2, !file.exists(args[2]))
stopifnot(unname(tools::md5sum(args[1]))=="cc37709732a1d5b65f73536563ebbf3e")
library(foreign)
d <- read.dta(args[1],convert.factors=FALSE)
expected <- strsplit("spell censor1 censor2 censor3 censor4 ui reprate logwage tenure disrate slack abolpos explose stateur houshead married female child ychild nonwhite age schlt12 schgt12 smsa bluecoll mining constr transp trade fire services pubadmin year85 year87 year89 midatl encen wncen southatl escen wscen mountain pacific"," ")[[1]]
stopifnot(identical(names(d),expected),nrow(d)==3343,all(vapply(d,is.numeric,logical(1))),
  all(is.finite(as.matrix(d))),sum(d$censor1)==1073,sum(d$spell)==20887,
  all(d$spell>0),all(as.matrix(d[paste0("censor",1:4)]) %in% c(0,1)),
  all(rowSums(d[paste0("censor",1:4)])<=1))
d <- data.frame(source_row=seq_len(nrow(d)),d,check.names=FALSE)
# Explicit17-digit round-trip strings preserve original promoted DTA float values.
encoded <- as.data.frame(lapply(d,function(x) sprintf("%.17g",x)),check.names=FALSE)
tmp <- tempfile(pattern=".duration-",tmpdir=dirname(args[2]))
tryCatch({
  write.table(encoded,tmp,sep=",",quote=FALSE,row.names=FALSE,col.names=TRUE)
  # R's decimal numeric parser can differ by1ULP from IEEE strtod. Verify text
  # publication here; the source audit checks exact numbers against DTA hex cells.
  back <- read.csv(tmp,check.names=FALSE,colClasses="character")
  stopifnot(identical(as.matrix(back),as.matrix(encoded)))
  if (!file.link(tmp,args[2])) stop("refusing output publication")
},finally=unlink(tmp))
cat("Prepared",nrow(d),"original DTA rows; foreign",as.character(packageVersion("foreign")),"\n")
