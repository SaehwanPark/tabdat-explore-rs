# Deterministic redistributable 12-firm panel including two all-zero firms.
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==1L, !file.exists(args[1]))
dir.create(args[1])
rows <- lapply(1:12,function(i) {
  logr <- vapply(0:9,function(t) .12*i+.17*t+.025*((i*t)%%7),numeric(1))
  pat <- vapply(0:9,function(t) if (i %in% c(3,9)) 0 else (i*3+t*2+i*t)%%7,numeric(1))
  c(i+2000,if(i==5) '.' else as.character(10+i%%5),i%%2,
    sprintf('%.5f',1+i/100),sum(pat),sprintf('%.5f',logr),pat)
})
writeLines(vapply(rows,paste,collapse=' ',FUN.VALUE=character(1)),file.path(args[1],'source.asc'))
