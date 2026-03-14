import OxFunc.FunctionCore
import OxFunc.LocaleFormat

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExcept [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def textMeta : FunctionMeta := {
  functionId := "FUNC.TEXT"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.textToText
  fecDependencyProfile := FecDependencyProfile.localeProfile
  surfaceFecDependencyProfile := FecDependencyProfile.composite
}

def evalTextNumber (profile : LocaleProfileId) (dateSystem : WorkbookDateSystem) (value : Rat) (code : String) : Except String String :=
  match renderWithCode profile dateSystem value code with
  | some rendered => .ok rendered
  | none => .error "#VALUE!"

def evalTextLogical (b : Bool) : String := if b then "TRUE" else "FALSE"

def evalTextTextFallback (profile : LocaleProfileId) (dateSystem : WorkbookDateSystem) (text code : String) : String :=
  match parseValueText profile dateSystem text with
  | some parsed => match renderWithCode profile dateSystem parsed code with
      | some rendered => rendered
      | none => text
  | none => text

theorem evalTextNumber_percent_seed :
    evalTextNumber .currentExcelHost .system1900 (1 / 8) "0%" = .ok "13%" := by
  native_decide

theorem evalTextNumber_date_seed :
    evalTextNumber .currentExcelHost .system1900 45325 "yyyy-mm-dd" = .ok "2024-02-03" := by
  native_decide

theorem evalTextLogical_true_seed :
    evalTextLogical true = "TRUE" := by
  native_decide

theorem evalTextTextFallback_nonnumeric_preserved :
    evalTextTextFallback .currentExcelHost .system1900 "x" "0" = "x" := by
  native_decide

theorem evalTextTextFallback_numeric_text_renders :
    evalTextTextFallback .currentExcelHost .system1900 "123" "0.00" = "123.00" := by
  native_decide

theorem textMeta_profiles :
    textMeta.kernelSignatureClass = KernelSignatureClass.textToText
    ∧ textMeta.fecDependencyProfile = FecDependencyProfile.localeProfile
    ∧ textMeta.surfaceFecDependencyProfile = FecDependencyProfile.composite := by
  simp [textMeta]

end OxFunc.Functions
