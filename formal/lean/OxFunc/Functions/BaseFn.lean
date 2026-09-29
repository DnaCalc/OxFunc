import OxFunc.FunctionCore
import OxFunc.CoercionPrimitives
namespace OxFunc.Functions
open OxFunc
def baseMeta : FunctionMeta := {
  functionId := "FUNC.BASE", arity := { min := 2, max := 3 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
/-- Finite numeric admission and integer preparation, aligned with BASE's
Value2 observations on build 20430/CV2. Rat is the exact finite-number substrate;
binary64 transport and decimal formatting are separate obligations. -/
def baseIntegerArguments (number radix : Rat) (length : Option Rat) : Option (Nat × Nat × Nat) :=
  let r := radix.num.tdiv (Int.ofNat radix.den)
  let p := length.getD 1
  let width := p.num.tdiv (Int.ofNat p.den)
  if number < 0 ∨ number ≥ 9007199254740992 ∨ r < 2 ∨ r > 36 ∨ p < 0 ∨ width > 255
  then none
  else some ((number.num.tdiv (Int.ofNat number.den)).toNat, r.toNat, width.toNat)

/-- Omitted width gives zero one digit; explicit zero width permits an empty
representation of zero. This distinction survives scalar coercion. -/
def baseZeroDigitCount (length : Option Rat) : Option Nat := do
  let (_, _, width) ← baseIntegerArguments 0 10 length
  return width

theorem baseIntegerArguments_bounds :
    baseIntegerArguments ((-9 : Rat) / 10) 10 none = none ∧
    baseIntegerArguments 9007199254740992 10 none = none ∧
    baseIntegerArguments 9007199254740991 36 (some 255) = some (9007199254740991, 36, 255) ∧
    baseIntegerArguments 1 10 (some 256) = none := by native_decide

theorem baseZeroDigitCount_omitted_vs_explicit :
    baseZeroDigitCount none = some 1 ∧
    baseZeroDigitCount (some 0) = some 0 ∧
    baseZeroDigitCount (some ((9 : Rat) / 10)) = some 0 := by native_decide

theorem baseIntegerArguments_fractional_positive :
    baseIntegerArguments ((319 : Rat) / 10) ((169 : Rat) / 10) (some ((48 : Rat) / 10)) =
      some (31, 16, 4) := by native_decide

theorem baseMeta_profiles :
    baseMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ baseMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ baseMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [baseMeta]
end OxFunc.Functions
