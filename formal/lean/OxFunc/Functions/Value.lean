import OxFunc.FunctionCore
import OxFunc.LocaleFormat

namespace OxFunc.Functions

open OxFunc

def valueMeta : FunctionMeta := {
  functionId := "FUNC.VALUE"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.localeProfile
  surfaceFecDependencyProfile := FecDependencyProfile.composite
}

def evalValueText (profile : LocaleProfileId) (dateSystem : WorkbookDateSystem) (text : String) : Except String Rat :=
  match parseValueText profile dateSystem text with
  | some n => .ok n
  | none => .error "#VALUE!"

theorem evalValueText_currentHost_grouped_numeric :
    evalValueText .currentExcelHost .system1900 "1 234.5" = .ok (2469 / 2) := by
  simp [evalValueText, parseValueText]

theorem evalValueText_currentHost_currency :
    evalValueText .currentExcelHost .system1900 "R1 234.57" = .ok (123457 / 100) := by
  simp [evalValueText, parseValueText]

theorem evalValueText_currentHost_percent :
    evalValueText .currentExcelHost .system1900 "12%" = .ok (12 / 100) := by
  simp [evalValueText, parseValueText]

theorem evalValueText_currentHost_iso_date :
    evalValueText .currentExcelHost .system1900 "2024-02-03" = .ok 45325 := by
  simp [evalValueText, parseValueText]

theorem evalValueText_currentHost_slash_date_rejected :
    evalValueText .currentExcelHost .system1900 "1/2/2024" = .error "#VALUE!" := by
  simp [evalValueText, parseValueText]

theorem valueMeta_profiles :
    valueMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ valueMeta.fecDependencyProfile = FecDependencyProfile.localeProfile
    ∧ valueMeta.surfaceFecDependencyProfile = FecDependencyProfile.composite := by
  simp [valueMeta]

end OxFunc.Functions
