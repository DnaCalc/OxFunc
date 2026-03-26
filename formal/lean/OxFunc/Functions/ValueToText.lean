import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def valueToTextMeta : FunctionMeta := {
  functionId := "FUNC.VALUETOTEXT"
  arity := { min := 1, max := 2 }
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

inductive ValueKind where
  | number (n : Rat)
  | text (s : String)
  | logical (b : Bool)
  | error (e : String)
  deriving DecidableEq, Repr

def valueToTextConcise : ValueKind → String
  | .number n => toString n
  | .text s => s
  | .logical true => "TRUE"
  | .logical false => "FALSE"
  | .error e => e

def valueToTextStrict : ValueKind → String
  | .number n => toString n
  | .text s => "\"" ++ s ++ "\""
  | .logical true => "TRUE"
  | .logical false => "FALSE"
  | .error e => "#" ++ e

theorem valueToText_number_same_both_modes (n : Rat) :
    valueToTextConcise (.number n) = valueToTextStrict (.number n) := by
  rfl

theorem valueToText_logical_true_concise :
    valueToTextConcise (.logical true) = "TRUE" := by rfl

theorem valueToText_text_strict_quotes :
    valueToTextStrict (.text "hello") = "\"hello\"" := by rfl

theorem valueToTextMeta_profiles :
    valueToTextMeta.hostInteraction = HostInteractionClass.none
    ∧ valueToTextMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ valueToTextMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ valueToTextMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [valueToTextMeta]

end OxFunc.Functions
