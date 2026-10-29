# Development-only inspectable balanced negative-correlation panel from source fixture.
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L,!file.exists(args[2]))
tokens <- scan(args[1],what=character(),quiet=TRUE)
stopifnot(length(tokens)==12L*25L)
src <- matrix(tokens,ncol=25L,byrow=TRUE)
for(i in 1:12) for(t in 0:4) {
  src[i,11+t] <- sprintf('%.2f',(i%%4)*.3+((t*2+i)%%5)*.15)
  src[i,21+t] <- as.character(c(1,1,2,3,3)[t+1]+as.integer(i%%5==0))
}
write.table(src,args[2],quote=FALSE,row.names=FALSE,col.names=FALSE)
