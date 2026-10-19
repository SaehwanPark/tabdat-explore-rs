# Development-only audited identity enrichment, not a runtime R dependency.
# Rscript prepare_person.R mma20p1count.asc randdata.dta mma20p1count_person.csv
# Verify the published source SHA256 checksums documented in README.md first.
main <- function(args) {
  stopifnot(length(args)==3L)
  values <- scan(args[1],quiet=TRUE)
  stopifnot(length(values)==20186L*18L)
  d <- as.data.frame(matrix(values,ncol=18,byrow=TRUE))
  names(d) <- c('MDU','LC','IDP','LPI','FMDE','PHYSLIM','NDISEASE','HLTHG','HLTHF',
    'HLTHP','LINC','LFAM','EDUCDEC','AGE','FEMALE','CHILD','FEMCHILD','BLACK')
  r <- foreign::read.dta(args[2],convert.factors=FALSE)
  keep <- !is.na(r$educdec)
  stopifnot(nrow(r)==20190L,identical(which(!keep),c(9746L,10153L,10400L,10859L)))
  original <- r[keep,]
  map <- c('mdvis','logc','idp','lpi','fmde','physlm','disea','hlthg','hlthf',
    'hlthp','linc','lfam','educdec','xage','female','child','fchild','black')
  # This audit is positional, but validates every author model cell and the
  # exact outcome/order before attaching the ORIGINAL IDs. It is not a key
  # inferred from the prepared row or a probabilistic join on rounded values.
  stopifnot(nrow(original)==nrow(d),!anyNA(original[map]),
    identical(d$MDU,as.numeric(original$mdvis)),
    all(vapply(seq_along(map),function(j)max(abs(d[[j]]-original[[map[j]]])),numeric(1))<5e-6),
    sum(d$MDU)==57746,sum(d$MDU==0)==6308,max(d$MDU)==77,
    !anyNA(original$zper),all(is.finite(original$zper)),
    all(original$zper==floor(original$zper)),all(abs(original$zper)<=2^53-1),
    identical(sort(unique(original$year)),as.numeric(1:5)),
    !anyDuplicated(original[c('zper','year')]),length(unique(original$zper))==5908L,
    identical(as.integer(table(original$year)),c(5638L,5574L,5545L,1715L,1714L)),
    identical(as.integer(table(as.integer(table(original$zper)))),c(265L,246L,3743L,70L,1584L)))
  prepared <- data.frame(prepared_row=seq_len(nrow(d)),original_row=which(keep),
    person_id=original$zper,study_year=original$year,d)
  target <- args[3]
  if (!dir.exists(dirname(target))) stop('output directory does not exist')
  stage <- tempfile(pattern='.tabdat-person-',tmpdir=dirname(target))
  on.exit(unlink(stage),add=TRUE)
  write.csv(prepared,stage,row.names=FALSE,quote=FALSE,na='')
  # Same-directory hard linking publishes only a complete file without
  # replacing files/directories/symlinks; no crash durability is promised.
  if (!file.link(stage,target)) stop('could not publish new output without replacement: ',target)
  cat('Prepared20,186 rows/5,908 original person IDs; unchanged ASCII model cells.\n')
  cat('Original row/person/year are audited source identities, NOT prepared positions.\n')
}
main(commandArgs(trailingOnly=TRUE))
