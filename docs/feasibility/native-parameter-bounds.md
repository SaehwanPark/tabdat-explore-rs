# Native packed-covariance parameter bounds (in progress)

Base c122ab6. Heckman already guards upstream signed `(tot*tot+tot)/2`, but source
inspection finds analogous signed `k*(k+1)/2` / `(k*k+k)/2` in qr_estimate.c:190,
estimate.c:908 and gretl_model.c:2616/2969/5214/5330. Checking only list length fits
c_int does not bound these intermediate products. Safe wrappers must reject the
nominal parameter count before native dispatch, independent of possible rank drop.

Bounded plan: pure checked count criterion `k*(k+1)<=i32::MAX` (max46340), shared
low-level guard and owned facade validation. Include extra native dispersion/scale
and two-equation parameters, special IV/survival lists; dynamic capped API is
already lower. Reject malformed oversized lists using small datasets/large repeated
indices before C; do not allocate real46341-square native matrices or invoke UB to
prove a bug. Pure threshold tests46340/46341/usize::MAX plus nominal-family guard/
failure recovery and all numeric fixtures/platform/policy/full unsafe inventory.

No numerical/tolerance/dependency change, no native C allocation/sanitizer/other
integer-product certification. Do not silently claim every source dimension/packed
index is safe from this one bound; other source arithmetic remains review work.
