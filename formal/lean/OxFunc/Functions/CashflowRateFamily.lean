import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def cashflowRateBaseMeta : FunctionMeta := {
  functionId := "FUNC.CASHFLOW_RATE_BASE"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def irrMeta : FunctionMeta := { cashflowRateBaseMeta with functionId := "FUNC.IRR", arity := { min := 1, max := 2 } }
def xnpvMeta : FunctionMeta := { cashflowRateBaseMeta with functionId := "FUNC.XNPV", arity := Arity.exact 3 }
def xirrMeta : FunctionMeta := { cashflowRateBaseMeta with functionId := "FUNC.XIRR", arity := { min := 2, max := 3 } }

/--
W24 Batch 12 packet-evidences the admitted numeric cashflow/date-vector slice
for `IRR`, `XNPV`, and `XIRR`. The bounded iterative solver and serial-date
interpretation live in Rust; this Lean file keeps metadata alignment explicit.
-/
theorem cashflowRateFamily_meta_profiles :
    irrMeta.arity = { min := 1, max := 2 }
    ∧ xnpvMeta.arity = Arity.exact 3
    ∧ xirrMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ xirrMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ xnpvMeta.threadSafety = ThreadSafetyClass.safePure := by
  simp [cashflowRateBaseMeta, irrMeta, xnpvMeta, xirrMeta]

theorem cashflowRateFamily_ids :
    irrMeta.functionId = "FUNC.IRR"
    ∧ xnpvMeta.functionId = "FUNC.XNPV"
    ∧ xirrMeta.functionId = "FUNC.XIRR" := by
  simp [irrMeta, xnpvMeta, xirrMeta]

end OxFunc.Functions
