import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def textCompatLocaleBase : FunctionMeta := {
  functionId := "FUNC.TEXT_COMPAT_LOCALE_BASE"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.none
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def ascMeta : FunctionMeta := { textCompatLocaleBase with functionId := "FUNC.ASC" }
def dbcsMeta : FunctionMeta := { textCompatLocaleBase with functionId := "FUNC.DBCS" }
def jisMeta : FunctionMeta := { textCompatLocaleBase with functionId := "FUNC.JIS" }

theorem textCompatLocale_profiles :
    ascMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ dbcsMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ jisMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [textCompatLocaleBase, ascMeta, dbcsMeta, jisMeta]

end OxFunc.Functions
