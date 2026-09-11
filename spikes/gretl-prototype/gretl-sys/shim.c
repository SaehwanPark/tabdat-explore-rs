/*
 * gretl_shim.c -- ownership shim for the TabDat libgretl feasibility spike.
 *
 * libgretl's estimation entry point `lsq` returns a large `MODEL` struct by
 * value whose fields point at C-allocated memory, and its `DATASET`/`MODEL`
 * structs are mutable C structures. Re-declaring those layouts in Rust would
 * couple the binding to the exact struct layout of a pinned C release.
 *
 * Instead this shim (compiled from the same pinned source tree, so its view of
 * the structs is always correct) owns all native objects and exposes only
 * opaque pointers plus scalar/array accessors. The Rust `gretl-sys` crate
 * wraps those opaque pointers in RAII types; no raw pointer or foreign struct
 * escapes the `gretl-sys` crate.
 *
 * This file is the single place that touches libgretl's C API.
 */

#include "libgretl.h"

#include <string.h>

/* ------------------------------------------------------------------ */
/* Process lifecycle                                                  */
/* ------------------------------------------------------------------ */

void gretl_shim_init(void)
{
    libgretl_init();
}

void gretl_shim_cleanup(void)
{
    libgretl_cleanup();
}

/* ------------------------------------------------------------------ */
/* Dataset                                                            */
/* ------------------------------------------------------------------ */

/* Create a dataset with `nvar` variables (variable 0 is the constant)
 * and `nobs` observations. Returns an opaque handle. */
void *gretl_shim_dataset_new(int nvar, int nobs)
{
    return (void *) create_new_dataset(nvar, nobs, 0);
}

/* Set the name of variable `i`. Names are truncated to gretl's VNAMELEN. */
void gretl_shim_dataset_set_varname(void *handle, int i, const char *name)
{
    DATASET *dset = (DATASET *) handle;
    size_t n = strlen(name);
    if (n >= (size_t) VNAMELEN) {
        n = (size_t) VNAMELEN - 1;
    }
    memcpy(dset->varname[i], name, n);
    dset->varname[i][n] = '\0';
}

/* Write one observation value. `var` is the variable index, `obs` the
 * 0-based observation index. */
void gretl_shim_dataset_set_value(void *handle, int var, int obs, double value)
{
    DATASET *dset = (DATASET *) handle;
    dset->Z[var][obs] = value;
}

void gretl_shim_dataset_free(void *handle)
{
    destroy_dataset((DATASET *) handle);
}

/* ------------------------------------------------------------------ */
/* OLS estimation                                                     */
/* ------------------------------------------------------------------ */

/* Run ordinary least squares.
 *
 * `list` is a gretl variable list: list[0] is the count of the entries that
 * follow, then the dependent variable, then 0 for the constant (if present),
 * then the regressor variable indices. `nlist` is the length of `list`.
 *
 * Returns an opaque MODEL handle owned by the shim; free it with
 * gretl_shim_model_free. The handle is valid even when estimation fails
 * (inspect gretl_shim_model_errcode). */
void *gretl_shim_ols(void *dset_handle, const int *list, int nlist)
{
    (void) nlist; /* list[0] carries the count; nlist is a sanity bound */
    DATASET *dset = (DATASET *) dset_handle;
    MODEL *model = gretl_model_new();
    if (model == NULL) {
        return NULL;
    }
    *model = lsq(list, dset, OLS, OPT_Z);
    return (void *) model;
}

/* ------------------------------------------------------------------ */
/* Result accessors                                                   */
/* ------------------------------------------------------------------ */

int gretl_shim_model_ncoeff(const void *handle)
{
    return ((const MODEL *) handle)->ncoeff;
}

int gretl_shim_model_nobs(const void *handle)
{
    return ((const MODEL *) handle)->nobs;
}

int gretl_shim_model_dfn(const void *handle)
{
    return ((const MODEL *) handle)->dfn;
}

int gretl_shim_model_dfd(const void *handle)
{
    return ((const MODEL *) handle)->dfd;
}

int gretl_shim_model_errcode(const void *handle)
{
    return ((const MODEL *) handle)->errcode;
}

double gretl_shim_model_coeff(const void *handle, int i)
{
    return ((const MODEL *) handle)->coeff[i];
}

double gretl_shim_model_sderr(const void *handle, int i)
{
    return ((const MODEL *) handle)->sderr[i];
}

/* (i, j) element of the coefficient covariance matrix. */
double gretl_shim_model_vcv(const void *handle, int i, int j)
{
    const MODEL *model = (const MODEL *) handle;
    return gretl_model_get_vcv_element(model, i, j, model->ncoeff);
}

/* Scalar fit statistics, selected by the SHIM_STAT_* codes. */
enum {
    SHIM_STAT_RSQ = 0,
    SHIM_STAT_ADJRSQ,
    SHIM_STAT_SIGMA,
    SHIM_STAT_ESS,
    SHIM_STAT_TSS,
    SHIM_STAT_FSTT,
    SHIM_STAT_LNL
};

double gretl_shim_model_scalar(const void *handle, int which)
{
    const MODEL *model = (const MODEL *) handle;
    switch (which) {
    case SHIM_STAT_RSQ:    return model->rsq;
    case SHIM_STAT_ADJRSQ: return model->adjrsq;
    case SHIM_STAT_SIGMA:  return model->sigma;
    case SHIM_STAT_ESS:    return model->ess;
    case SHIM_STAT_TSS:    return model->tss;
    case SHIM_STAT_FSTT:   return model->fstt;
    case SHIM_STAT_LNL:    return model->lnL;
    default:               return 0.0;
    }
}

void gretl_shim_model_free(void *handle)
{
    gretl_model_free((MODEL *) handle);
}
