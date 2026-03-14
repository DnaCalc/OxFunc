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

def dollarMeta : FunctionMeta := {
  functionId := "FUNC.DOLLAR"
  arity := { min := 1, max := 2 }
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

def evalDollarSeed (profile : LocaleProfileId) (value : Rat) (decimals : Int) : Except String String :=
  match renderCurrency profile value decimals with
  | some rendered => .ok rendered
  | none => .error "#VALUE!"

theorem evalDollar_positive_seed :
    evalDollarSeed .currentExcelHost (1234567 / 1000) 2 = .ok "R1 234.57" := by
  native_decide

theorem evalDollar_negative_seed :
    evalDollarSeed .currentExcelHost (-1234567 / 1000) 2 = .ok "-R1 234.57" := by
  native_decide

theorem evalDollar_blank_zero_seed :
    evalDollarSeed .currentExcelHost 0 2 = .ok "R0.00" := by
  native_decide

theorem dollarMeta_profiles :
    dollarMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ dollarMeta.fecDependencyProfile = FecDependencyProfile.localeProfile
    ∧ dollarMeta.surfaceFecDependencyProfile = FecDependencyProfile.composite := by
  simp [dollarMeta]

end OxFunc.Functions
