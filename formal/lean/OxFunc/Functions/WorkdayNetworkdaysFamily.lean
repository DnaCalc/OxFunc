import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def workdayNetworkdaysBase : FunctionMeta := {
  functionId := "FUNC.WORKDAY_NETWORKDAYS_BASE"
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

def workdayMeta : FunctionMeta := {
  workdayNetworkdaysBase with functionId := "FUNC.WORKDAY", arity := { min := 2, max := 3 }
}

def workdayIntlMeta : FunctionMeta := {
  workdayNetworkdaysBase with functionId := "FUNC.WORKDAY.INTL", arity := { min := 2, max := 4 }
}

def networkdaysMeta : FunctionMeta := {
  workdayNetworkdaysBase with functionId := "FUNC.NETWORKDAYS", arity := { min := 2, max := 3 }
}

def networkdaysIntlMeta : FunctionMeta := {
  workdayNetworkdaysBase with functionId := "FUNC.NETWORKDAYS.INTL", arity := { min := 2, max := 4 }
}

theorem workdayNetworkdays_profiles :
    workdayMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ workdayIntlMeta.arity.max = 4
    ∧ networkdaysMeta.functionId = "FUNC.NETWORKDAYS"
    ∧ networkdaysIntlMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [workdayNetworkdaysBase, workdayMeta, workdayIntlMeta, networkdaysMeta, networkdaysIntlMeta]

end OxFunc.Functions
