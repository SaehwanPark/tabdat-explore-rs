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
#include "libset.h"
#include "discrete.h"

#include <string.h>

/* ------------------------------------------------------------------ */
/* Process lifecycle                                                  */
/* ------------------------------------------------------------------ */

void gretl_shim_init(void)
{
    libgretl_init();
#ifdef TD_COUNT_PLUGIN_PATH
    /* plugins.c concatenates path + basename into MAXLEN storage. Keep the
     * compile-time, build-owned path bounded, including separator and suffix. */
    _Static_assert(sizeof(TD_COUNT_PLUGIN_PATH) + sizeof("/poisson.so") < MAXLEN,
                   "plugin build path exceeds native path buffer");
    set_gretl_plugin_path(TD_COUNT_PLUGIN_PATH);
#endif
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
void *gretl_shim_ols(void *dset_handle, const int *list, int nlist,
                     int covariance, int cluster_var)
{
    DATASET *dset = (DATASET *) dset_handle;
    if (dset == NULL || list == NULL || nlist < 3 || list[0] != nlist - 1) {
        return NULL;
    }
    MODEL *model = gretl_model_new();
    if (model == NULL) {
        return NULL;
    }
    int saved_hc = libset_get_int(HC_VERSION);
    gretlopt opt = OPT_Z;
    if (covariance == 1) {
        libset_set_int(HC_VERSION, 1);
        opt |= OPT_R;
    } else if (covariance == 2) {
        if (cluster_var <= 0 || cluster_var >= dset->v) {
            model->errcode = E_DATA;
            return model;
        }
        int err = set_optval_string(OLS, OPT_C, dset->varname[cluster_var]);
        if (err) {
            model->errcode = err;
            clear_stored_options_for_command(OLS);
            return model;
        }
        opt |= OPT_C;
    }
    *model = lsq(list, dset, OLS, opt);
    /* Classical lsq may leave VCV packed storage lazy; materialize it while
     * the model is still exclusively owned under the Rust native-call lock. */
    if (!model->errcode && model->vcv == NULL) {
        model->errcode = makevcv(model, model->sigma);
    }
    libset_set_int(HC_VERSION, saved_hc);
    if (covariance == 2) {
        clear_stored_options_for_command(OLS);
    }
    return (void *) model;
}

/* ------------------------------------------------------------------ */
/* Result accessors                                                   */
/* ------------------------------------------------------------------ */

/* Binary entry points return MODEL by value just like lsq. The shim owns
 * the copied result, and callers use the same checked accessors/destructor. */
void *gretl_shim_binary(void *dset_handle, const int *list, int nlist, int probit)
{
    if (dset_handle == NULL || list == NULL || nlist < 3 || list[0] != nlist - 1) {
        return NULL;
    }
    MODEL *model = gretl_model_new();
    if (model == NULL) {
        return NULL;
    }
    if (probit) {
        *model = binary_probit(list, (DATASET *) dset_handle, OPT_NONE, NULL);
    } else {
        *model = binary_logit(list, (DATASET *) dset_handle, OPT_NONE, NULL);
    }
    return model;
}

#ifdef TD_COUNT_PLUGIN_PATH
void *gretl_shim_count(void *dset_handle, const int *list, int nlist, int nb2)
{
    if (dset_handle == NULL || list == NULL || nlist < 3 || list[0] != nlist - 1) {
        return NULL;
    }
    MODEL *model = gretl_model_new();
    if (model == NULL) {
        return NULL;
    }
    /* count_model owns the normal OLS setup and native plugin dispatch. The
     * plugin links the same shared core and is retained until process exit. */
    *model = count_model(list, nb2 ? NEGBIN : POISSON, (DATASET *) dset_handle, OPT_NONE, NULL);
    return model;
}
#endif

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
