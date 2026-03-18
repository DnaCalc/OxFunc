import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def couponBase : FunctionMeta := {
  functionId := "FUNC.COUPON_BASE"
  arity := { min := 3, max := 4 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def coupdaybsMeta : FunctionMeta := { couponBase with functionId := "FUNC.COUPDAYBS" }
def coupdaysMeta : FunctionMeta := { couponBase with functionId := "FUNC.COUPDAYS" }
def coupdaysncMeta : FunctionMeta := { couponBase with functionId := "FUNC.COUPDAYSNC" }
def coupncdMeta : FunctionMeta := { couponBase with functionId := "FUNC.COUPNCD" }
def coupnumMeta : FunctionMeta := { couponBase with functionId := "FUNC.COUPNUM" }
def couppcdMeta : FunctionMeta := { couponBase with functionId := "FUNC.COUPPCD" }

/--
W24 Batch 13 packet-evidences the admitted regular coupon-schedule slice for
the six coupon functions. The detailed schedule and date-boundary logic live in
Rust; this Lean file keeps metadata alignment explicit.
-/
theorem coupon_profiles :
    coupdaybsMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ coupdaysMeta.arity.min = 3
    ∧ coupdaysncMeta.functionId = "FUNC.COUPDAYSNC"
    ∧ coupncdMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ coupnumMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ couppcdMeta.fecDependencyProfile = FecDependencyProfile.none := by
  simp [couponBase, coupdaybsMeta, coupdaysMeta, coupdaysncMeta, coupncdMeta, coupnumMeta, couppcdMeta]

end OxFunc.Functions

