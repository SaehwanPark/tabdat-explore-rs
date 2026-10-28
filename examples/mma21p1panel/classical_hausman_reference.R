# Independent plm/matrix reference for the original MOM scalar Hausman test.
# Rscript classical_hausman_reference.R MOM.dat fresh-output-directory
options(digits=17)
args <- commandArgs(trailingOnly=TRUE); stopifnot(length(args)==2,!file.exists(args[2]))
v <- scan(args[1],quiet=TRUE); stopifnot(length(v)==5320*8,all(is.finite(v)))
d <- as.data.frame(matrix(v,ncol=8,byrow=TRUE))
names(d) <- c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
stopifnot(identical(d$id,rep(as.numeric(1:532),each=10)),identical(d$year,rep(as.numeric(1979:1988),532)))
fe <- plm::plm(lnhr~lnwg,data=d,index=c('id','year'),model='within')
re <- plm::plm(lnhr~lnwg,data=d,index=c('id','year'),model='random',random.method='swar')
bf <- unname(coef(fe)[1]); br <- unname(coef(re)['lnwg'])
vf <- unname(vcov(fe)[1,1]); vr <- unname(vcov(re)['lnwg','lnwg'])
delta <- bf-br; variance <- vf-vr; stopifnot(variance>0)
statistic <- delta^2/variance; probability <- pchisq(statistic,1,lower.tail=FALSE)
phtest <- plm::phtest(fe,re)
stopifnot(abs(unname(phtest$statistic)-statistic)<1e-10,abs(phtest$p.value-probability)<1e-12)
fields <- c(fe_coefficient=bf,re_coefficient=br,fe_variance=vf,re_variance=vr,
 difference=delta,variance_difference=variance,statistic=statistic,df=1,p_value=probability,
 observation_count=nrow(d),group_count=length(unique(d$id)),group_size=10)
dir.create(args[2]); write.csv(data.frame(field=names(fields),value=unname(fields)),file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'plm',as.character(packageVersion('plm')),'\n'); print(fields)
