# Development-only independent one-way person-clustered NB2 reference.
# Synthetic: Rscript nb2_cluster_reference.R > nb2_cluster_reference.csv
# Empirical: Rscript nb2_cluster_reference.R mma20p1count.asc randdata.dta \
#   /tmp/tabdat-283-nb2-person-predictions.csv > /tmp/tabdat-283-nb2-person-reference.csv
# The empirical files are read directly from the original ASC and DTA inputs;
# only the synthetic scalar fixture is checked in.
options(digits=17,OutDec='.')
invisible(Sys.setlocale('LC_NUMERIC','C'))
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args) %in% c(0L,3L))
rows <- list()
emit <- function(case,quantity,i=0L,j=0L,value) {
  rows[[length(rows)+1L]] <<- data.frame(case=case,quantity=quantity,
    i=as.integer(i),j=as.integer(j),value=as.numeric(value),
    stringsAsFactors=FALSE)
}

# This is the full joint NB2 score and observed information in (beta, lnalpha).
# In particular, the last row/column is not the conditional MASS/Fisher bread.
evaluate <- function(par,X,y) {
  k <- ncol(X); r <- exp(-par[k+1L]); eta <- drop(X %*% par[seq_len(k)])
  mu <- exp(eta)
  sr <- digamma(y+r)-digamma(r)+log(r/(r+mu))+1-(r+y)/(r+mu)
  scores <- cbind(X*(r*(y-mu)/(r+mu)),lnalpha=-r*sr)
  cross <- drop(crossprod(X,r*mu*(y-mu)/(r+mu)^2))
  iaa <- sum(-r*sr-r^2*(trigamma(y+r)-trigamma(r)+1/r-1/(r+mu)+(y-mu)/(r+mu)^2))
  information <- rbind(
    cbind(crossprod(X,X*(r*mu*(r+y)/(r+mu)^2)),cross),c(cross,iaa))
  list(ll=sum(dnbinom(y,size=r,mu=mu,log=TRUE)),score=colSums(scores),
    information=information,scores=scores,mu=mu,eta=eta)
}
polish <- function(par,X,y) {
  for (iteration in seq_len(50L)) {
    e <- evaluate(par,X,y)
    step <- solve(e$information,e$score)
    if (max(abs(step)) < 1e-14) {
      return(list(par=unname(par),e=e,iterations=iteration-1L,
        final_step=unname(step)))
    }
    par <- unname(par+step)
  }
  stop('reference did not converge')
}
sha256_file <- function(path) {
  for (spec in list(c('shasum','-a','256'),c('sha256sum'))) {
    command <- spec[[1L]]
    if (!nzchar(Sys.which(command))) next
    output <- suppressWarnings(system2(command,c(spec[-1L],shQuote(path)),
      stdout=TRUE,stderr=TRUE))
    if (!is.null(attr(output,'status')) && attr(output,'status')!=0L) next
    digest <- sub('[[:space:]].*$','',output[[1L]])
    if (grepl('^[[:xdigit:]]{64}$',digest)) return(tolower(digest))
  }
  stop('could not calculate a SHA-256 input checksum')
}

reference <- function(case,y,predictors,groups,intercept=TRUE,
                      metadata=NULL,predictions_path=NULL) {
  X <- as.matrix(predictors)
  if (intercept) X <- cbind(intercept=1,X)
  n <- length(y); k <- ncol(X); p <- k+1L
  stopifnot(length(groups)==n,n>p,all(is.finite(y)),all(y>=0 & y==floor(y)),
    max(y)<=100000,all(is.finite(X)),qr(X)$rank==k,
    !anyNA(groups),length(unique(groups))>=2L)
  frame <- data.frame(y=y,predictors)
  formula <- if (ncol(predictors)==0L) y~1 else if (intercept) y~. else y~.-1
  initial <- MASS::glm.nb(formula,data=frame,
    control=glm.control(epsilon=1e-13,maxit=100))
  stopifnot(initial$converged)
  fitted <- polish(c(unname(coef(initial)),lnalpha=-log(initial$theta)),X,y)
  par <- unname(fitted$par); e <- fitted$e
  alpha <- exp(par[p]); r <- 1/alpha; B <- solve(e$information)
  G <- length(unique(groups)); correction <- G/(G-1)
  group_factor <- factor(groups,levels=unique(groups))
  group_scores <- rowsum(e$scores,group=group_factor,reorder=FALSE)
  split_scores <- do.call(rbind,lapply(split(seq_len(n),group_factor),
    function(index) colSums(e$scores[index,,drop=FALSE])))
  stopifnot(nrow(group_scores)==G,
    max(abs(unname(group_scores)-unname(split_scores)))<1e-12,
    max(abs(colSums(group_scores)-e$score))<1e-10)

  # Check group scores independently by differentiating each grouped log
  # likelihood in normalized joint coordinates.
  scales <- c(sqrt(colSums(X^2)),sqrt(n))
  normalized_group_score <- function(index) {
    t <- par*scales; h <- 1e-5
    vapply(seq_len(p),function(j) {
      up <- down <- t; up[j] <- up[j]+h; down[j] <- down[j]-h
      grouped_ll <- function(point) {
        candidate <- point/scales
        candidate_eta <- drop(X[index,,drop=FALSE] %*% candidate[seq_len(k)])
        sum(dnbinom(y[index],size=exp(-candidate[p]),
          mu=exp(candidate_eta),log=TRUE))
      }
      (grouped_ll(up)-grouped_ll(down))/(2*h)
    },numeric(1))
  }
  numerical_group_scores <- do.call(rbind,lapply(
    split(seq_len(n),group_factor),normalized_group_score))
  analytic_group_scores <- sweep(unname(group_scores),2L,scales,'/')
  group_score_discrepancy <- max(abs(numerical_group_scores-analytic_group_scores))
  stopifnot(group_score_discrepancy<1e-7)

  meat <- crossprod(group_scores)
  # Independently accumulate the outer products after grouping, rather than
  # substituting row scores or summing row-level outer products.
  outer_sum <- Reduce(`+`,lapply(seq_len(G),function(g)
    tcrossprod(group_scores[g,])))
  outer_discrepancy <- max(abs(meat-outer_sum))
  outer_relative_discrepancy <- outer_discrepancy/max(1,max(abs(meat)))
  stopifnot(outer_relative_discrepancy<100*.Machine$double.eps)
  V <- correction*B %*% meat %*% B
  stopifnot(all(is.finite(V)),max(abs(V-t(V)))<1e-10,
    all(diag(V)>=-1e-12))
  if (G==n) {
    row_robust_law <- n/(n-1)*B %*% crossprod(e$scores) %*% B
    stopifnot(max(abs(V-row_robust_law))<1e-12)
  }

  # Retain the established numerical differentiation check for the analytic
  # observed information, including beta/lnalpha cross-curvature.
  scaled_gradient <- function(t) evaluate(t/scales,X,y)$score/scales
  t <- par*scales; h <- 1e-4
  numeric_I <- -sapply(seq_len(p),function(j) {
    up <- down <- t; up[j] <- up[j]+h; down[j] <- down[j]-h
    (scaled_gradient(up)-scaled_gradient(down))/(2*h)
  })
  expected_I <- e$information/outer(scales,scales)
  curvature_discrepancy <- max(abs(numeric_I-expected_I))
  min_scaled_eigenvalue <- min(eigen(expected_I,symmetric=TRUE,
    only.values=TRUE)$values)
  stopifnot(curvature_discrepancy<1e-7,min_scaled_eigenvalue>0)

  # The restricted NB null reestimates dispersion; its LR is descriptive only
  # under clustering and intentionally has no reference p-value.
  if (intercept) {
    null_X <- matrix(1,n,1)
    null_start <- c(log(mean(y)),par[p])
  } else {
    null_X <- matrix(numeric(0),n,0)
    optimum <- optimize(function(a)sum(dnbinom(y,size=exp(-a),mu=1,log=TRUE)),
      c(-12,12),maximum=TRUE,tol=1e-12)
    null_start <- optimum$maximum
  }
  null <- polish(null_start,null_X,y)
  null_ll <- null$e$ll
  lr <- max(0,2*(e$ll-null_ll)); q <- k-as.integer(intercept)
  deviance <- sum(2*(ifelse(y==0,0,y*log(ifelse(y==0,1,y)/e$mu))-
    (y+r)*log((y+r)/(e$mu+r))))
  pearson <- sum((y-e$mu)^2/(e$mu+alpha*e$mu^2))
  slope <- if (q>0L) seq.int(1L+as.integer(intercept),k) else integer(0)
  slope_covariance <- if (q>0L) V[slope,slope,drop=FALSE] else matrix(numeric(0),0,0)
  wald_available <- q>0L && q<G && qr(slope_covariance)$rank==q
  wald <- if (wald_available) {
    drop(t(par[slope]) %*% solve(slope_covariance,par[slope]))
  } else NA_real_
  wald_p <- if (wald_available) pchisq(wald,q,lower.tail=FALSE) else NA_real_
  fields <- c(observation_count=n,zero_count=sum(y==0),count_sum=sum(y),
    cluster_group_count=G,cluster_correction=correction,
    log_likelihood=e$ll,null_log_likelihood=null_ll,
    null_alpha=exp(tail(null$par,1)),likelihood_ratio=lr,
    likelihood_ratio_df=q,likelihood_ratio_p_value=NA_real_,
    mcfadden_r_squared=1-e$ll/null_ll,residual_deviance=deviance,
    pearson_chi_square=pearson,df_residual=n-p,
    pearson_dispersion=pearson/(n-p),alpha=alpha,log_alpha=par[p],
    alpha_standard_error=alpha*sqrt(V[p,p]),
    log_alpha_standard_error=sqrt(V[p,p]),
    alpha_confidence_lower=exp(par[p]-qnorm(.975)*sqrt(V[p,p])),
    alpha_confidence_upper=exp(par[p]+qnorm(.975)*sqrt(V[p,p])),
    log_alpha_confidence_lower=par[p]-qnorm(.975)*sqrt(V[p,p]),
    log_alpha_confidence_upper=par[p]+qnorm(.975)*sqrt(V[p,p]),
    wald_statistic=wald,wald_p_value=wald_p)
  for (name in names(fields)) emit(case,name,value=fields[[name]])

  normal <- function(prefix,i,value,variance) {
    value <- unname(value); se <- sqrt(max(0,variance))
    f <- c(value=value,standard_error=se,
      statistic=if(se>0)value/se else NA_real_,
      p_value=if(se>0)2*pnorm(-abs(value/se)) else NA_real_,
      confidence_lower=if(se>0)value-qnorm(.975)*se else NA_real_,
      confidence_upper=if(se>0)value+qnorm(.975)*se else NA_real_)
    for (name in names(f)) emit(case,paste0(prefix,name),i,value=f[[name]])
  }
  for (i in seq_len(k)) normal('coefficient_',i-1L,par[i],V[i,i])
  for (i in seq_len(p)) for (j in seq_len(p))
    emit(case,'covariance',i-1L,j-1L,V[i,j])
  for (column in seq_len(q)) {
    j <- column+as.integer(intercept)
    gradient <- c(par[j]*colMeans(X*e$mu),0)
    gradient[j] <- gradient[j]+mean(e$mu)
    normal('effect_',column-1L,par[j]*mean(e$mu),
      drop(t(gradient)%*%V%*%gradient))
  }
  if (is.null(predictions_path)) for (i in seq_len(n)) {
    emit(case,'mean',i-1L,value=e$mu[i]); emit(case,'xb',i-1L,value=e$eta[i])
  }
  if (!is.null(predictions_path)) {
    stopifnot(nrow(metadata)==n)
    predictions <- data.frame(metadata,MDU=y,predictors,mu=e$mu,xb=e$eta,
      check.names=FALSE)
    write.csv(predictions,predictions_path,row.names=FALSE,quote=FALSE,na='NA')
  }
  scaled_step <- max(abs(fitted$final_step*scales))
  scaled_score <- max(abs(e$score/scales))
  message(sprintf(paste0('%s: G=%d correction=%.17g logalpha_se=%.17g ',
    'wald=%s wald_p=%s polish_iterations=%d scaled_score=%.4g ',
    'scaled_step=%.4g curvature_max_abs=%.4g group_score_max_abs=%.4g ',
    'min_scaled_information_eigenvalue=%.4g'),case,G,correction,
    sqrt(V[p,p]),format(wald,digits=17),format(wald_p,digits=17),
    fitted$iterations,scaled_score,scaled_step,curvature_discrepancy,
    group_score_discrepancy,min_scaled_eigenvalue))
}

if (length(args)==3L) {
  expected_sha <- c(
    'c8b1eee8ae1f637f4d4e4c6aa6b2dcbf2b8aa98f276b9369adcb4f02a528c58a',
    '8364cf9e53e32c794bcf0313a1590adcedf4e0526ba792884c7978cde169a628')
  stopifnot(identical(unname(vapply(args[1:2],sha256_file,character(1))),
    expected_sha))
  values <- scan(args[1],quiet=TRUE)
  stopifnot(length(values)==20186L*18L)
  d <- as.data.frame(matrix(values,ncol=18,byrow=TRUE))
  names(d) <- c('MDU','LC','IDP','LPI','FMDE','PHYSLIM','NDISEASE','HLTHG','HLTHF',
    'HLTHP','LINC','LFAM','EDUCDEC','AGE','FEMALE','CHILD','FEMCHILD','BLACK')
  r <- foreign::read.dta(args[2],convert.factors=FALSE)
  keep <- !is.na(r$educdec)
  stopifnot(nrow(r)==20190L,
    identical(which(!keep),c(9746L,10153L,10400L,10859L)))
  selected <- r[keep,]
  map <- c('mdvis','logc','idp','lpi','fmde','physlm','disea','hlthg','hlthf',
    'hlthp','linc','lfam','educdec','xage','female','child','fchild','black')
  stopifnot(nrow(selected)==nrow(d),
    identical(d$MDU,as.numeric(selected$mdvis)),
    all(vapply(seq_along(map),function(j)
      max(abs(d[[j]]-selected[[map[j]]])),numeric(1))<5e-6),
    sum(d$MDU)==57746,sum(d$MDU==0)==6308,max(d$MDU)==77,
    !anyNA(selected$zper),all(is.finite(selected$zper)),
    all(selected$zper==floor(selected$zper)),
    identical(sort(unique(selected$year)),as.numeric(1:5)),
    !anyDuplicated(selected[c('zper','year')]),
    length(unique(selected$zper))==5908L,
    identical(as.integer(table(selected$year)),c(5638L,5574L,5545L,1715L,1714L)),
    identical(as.integer(table(as.integer(table(selected$zper)))),
      c(265L,246L,3743L,70L,1584L)))
  metadata <- data.frame(prepared_row=seq_len(nrow(d)),original_row=which(keep),
    person_id=selected$zper,study_year=selected$year)
  package_versions <- c(R=as.character(getRversion()),
    MASS=as.character(packageVersion('MASS')),
    foreign=as.character(packageVersion('foreign')))
  message(sprintf('versions: %s; LC_ALL=%s; LC_COLLATE=%s; LC_NUMERIC=%s',
    paste(names(package_versions),package_versions,collapse=' '),
    Sys.getenv('LC_ALL'),Sys.getlocale('LC_COLLATE'),Sys.getlocale('LC_NUMERIC')))
  reference('mma20p1count_person',d$MDU,d[-1],metadata$person_id,
    metadata=metadata,predictions_path=args[3])
} else {
  package_versions <- c(R=as.character(getRversion()),
    MASS=as.character(packageVersion('MASS')))
  message(sprintf('versions: %s; LC_ALL=%s; LC_COLLATE=%s; LC_NUMERIC=%s',
    paste(names(package_versions),package_versions,collapse=' '),
    Sys.getenv('LC_ALL'),Sys.getlocale('LC_COLLATE'),Sys.getlocale('LC_NUMERIC')))
  x <- seq(-2,2,length.out=36); z <- rep(c(0,1,.4),12)
  y <- c(0,0,2,0,0,3,0,3,1,3,8,3,0,0,0,2,1,2,0,0,2,1,1,2,0,5,4,1,
    1,0,0,2,2,0,1,0)
  predictors <- data.frame(x=x,z=z)
  groups <- rep(seq_len(6),6)
  reference('multiple',y,predictors,groups)
  reference('noconstant',y,predictors,groups,FALSE)
  reference('intercept',y,matrix(numeric(0),36,0),groups)
  reference('singleton',y,predictors,seq_len(36))
}
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE,na='NA')
