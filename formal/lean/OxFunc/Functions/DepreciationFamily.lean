import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def depreciationBase : FunctionMeta := {
  functionId := "FUNC.DEPRECIATION_BASE"
  arity := Arity.exact 1
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

def slnMeta : FunctionMeta := { depreciationBase with functionId := "FUNC.SLN", arity := Arity.exact 3 }
def sydMeta : FunctionMeta := { depreciationBase with functionId := "FUNC.SYD", arity := Arity.exact 4 }
def dbMeta : FunctionMeta := { depreciationBase with functionId := "FUNC.DB", arity := { min := 4, max := 5 } }
def ddbMeta : FunctionMeta := { depreciationBase with functionId := "FUNC.DDB", arity := { min := 4, max := 5 } }
def vdbMeta : FunctionMeta := { depreciationBase with functionId := "FUNC.VDB", arity := { min := 5, max := 7 } }

theorem depreciation_family_profiles :
    slnMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ sydMeta.arity = Arity.exact 4
    ∧ dbMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ ddbMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ vdbMeta.arity.max = 7 := by
  simp [depreciationBase, slnMeta, sydMeta, dbMeta, ddbMeta, vdbMeta]

end OxFunc.Functions
