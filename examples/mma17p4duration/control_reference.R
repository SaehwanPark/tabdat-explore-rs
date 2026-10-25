# Regenerate numerical control, not replacement empirical data or a coverage study.
# Rscript examples/mma17p4duration/control_reference.R new_fixture_directory
args<-commandArgs(TRUE)
stopifnot(length(args)==1,!file.exists(args[1]))
set.seed(17042005)
n<-400
names<-strsplit("spell censor1 censor2 censor3 censor4 ui reprate logwage tenure disrate slack abolpos explose stateur houshead married female child ychild nonwhite age schlt12 schgt12 smsa bluecoll mining constr transp trade fire services pubadmin year85 year87 year89 midatl encen wncen southatl escen wscen mountain pacific"," ")[[1]]
d<-as.data.frame(matrix(0,n,length(names)));colnames(d)<-names
status<-sample(0:4,n,replace=TRUE,prob=c(.04,.32,.1,.17,.37));status[1]<-0
for(i in 1:4)d[[paste0("censor",i)]]<-as.numeric(status==i)
d$spell<-sample(1:28,n,replace=TRUE);d$ui<-rbinom(n,1,.5)
d$reprate<-sample(1:7,n,replace=TRUE)/8;d$disrate<-sample(0:3,n,replace=TRUE)/8
d$logwage<-sample(32:64,n,replace=TRUE)/8;d$tenure<-sample(1:20,n,replace=TRUE)
d$stateur<-sample(24:80,n,replace=TRUE)/8;d$age<-sample(20:60,n,replace=TRUE)
for(name in c("slack","abolpos","explose","houshead","married","female","child","ychild","nonwhite","smsa","bluecoll"))d[[name]]<-rbinom(n,1,.5)
for(group in list(c("schlt12","schgt12"),c("mining","constr","transp","trade","fire","services","pubadmin"),
  c("year85","year87","year89"),c("midatl","encen","wncen","southatl","escen","wscen","mountain","pacific"))) {
  category<-sample(0:length(group),n,replace=TRUE)
  for(i in seq_along(group))d[[group[i]]]<-as.numeric(category==i)
}
d<-data.frame(source_row=seq_len(n),d,check.names=FALSE)
stopifnot(dir.create(args[1]))
write.table(as.data.frame(lapply(d,function(x)sprintf("%.17g",x)),check.names=FALSE),
  file.path(args[1],"input.csv"),sep=",",quote=FALSE,row.names=FALSE)
script<-sub("control_reference.R$","reference.R",sub("^--file=","",grep("^--file=",commandArgs(),value=TRUE)))
stopifnot(system2(file.path(R.home("bin"),"Rscript"),c(shQuote(script),shQuote(file.path(args[1],"input.csv")),
  shQuote(file.path(args[1],"reference"))))==0)
