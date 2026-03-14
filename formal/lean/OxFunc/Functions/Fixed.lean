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

def fixedMeta : FunctionMeta := {
  functionId := "FUNC.FIXED"
  arity := { min := 1, max := 3 }
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

def evalFixedSeed (profile : LocaleProfileId) (value : Rat) (decimals : Int) (noCommas : Bool) : Except String String :=
  match renderFixed profile value decimals noCommas with
  | some rendered => .ok rendered
  | none => .error "#VALUE!"

theorem evalFixed_grouped_seed :
    evalFixedSeed .currentExcelHost (1234567 / 1000) 2 false = .ok "1 234.57" := by
  native_decide

theorem evalFixed_nocommas_seed :
    evalFixedSeed .currentExcelHost (1234567 / 1000) 2 true = .ok "1234.57" := by
  native_decide

theorem evalFixed_negative_seed :
    evalFixedSeed .currentExcelHost (-1234567 / 1000) 2 false = .ok "-1 234.57" := by
  native_decide

theorem fixedMeta_profiles :
    fixedMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ fixedMeta.fecDependencyProfile = FecDependencyProfile.localeProfile
    ∧ fixedMeta.surfaceFecDependencyProfile = FecDependencyProfile.composite := by
  simp [fixedMeta]

end OxFunc.Functions
