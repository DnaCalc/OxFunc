import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def financialTimeValuePureMeta (functionId : String) : FunctionMeta := {
  functionId := functionId
  arity := { min := 3, max := 5 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def rateMeta : FunctionMeta :=
  { (financialTimeValuePureMeta "FUNC.RATE") with arity := { min := 3, max := 6 } }

def irregularFinancialMeta (functionId : String) : FunctionMeta :=
  { (financialTimeValuePureMeta functionId) with arity := { min := 2, max := 3 } }

/--
W24 Batch 11 packet-evidences the admitted scalar/sequence financial family while
Lean remains at the metadata/binding alignment layer rather than re-encoding the
full numeric solver family.
-/
theorem financial_time_value_family_profiles :
    (financialTimeValuePureMeta "FUNC.PV").hostInteraction = HostInteractionClass.none
    ∧ (financialTimeValuePureMeta "FUNC.PMT").threadSafety = ThreadSafetyClass.safePure
    ∧ rateMeta.arity = { min := 3, max := 6 }
    ∧ (irregularFinancialMeta "FUNC.MIRR").surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [financialTimeValuePureMeta, rateMeta, irregularFinancialMeta]

end OxFunc.Functions
