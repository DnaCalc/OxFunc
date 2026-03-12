import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptRound [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def roundMeta : FunctionMeta := {
  functionId := "FUNC.ROUND"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.unaryNumericScalarOnly
  kernelSignatureClass := KernelSignatureClass.numsToNum
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def natPow10 : Nat → Nat
  | 0 => 1
  | n + 1 => natPow10 n * 10

def ratPow10 (n : Nat) : Rat := (natPow10 n : Rat)

def truncTowardZeroInt (q : Rat) : Int :=
  if q.num < 0 then
    -Int.ediv (-q.num) q.den
  else
    Int.ediv q.num q.den

def roundHalfAwayFromZeroInt (q : Rat) : Int :=
  if q < 0 then
    -truncTowardZeroInt ((-q) + (1 / 2 : Rat))
  else
    truncTowardZeroInt (q + (1 / 2 : Rat))

def truncateDigitsTowardZero (digits : Rat) : Int :=
  truncTowardZeroInt digits

def roundKernel (n : Rat) (digits : Int) : Rat :=
  if digits ≥ 0 then
    let factor := ratPow10 digits.toNat
    (roundHalfAwayFromZeroInt (n * factor) : Rat) / factor
  else
    let factor := ratPow10 (-digits).toNat
    (roundHalfAwayFromZeroInt (n / factor) : Rat) * factor

def evalRoundPrepared (value digits : CoercionInput) : Except CoercionError Rat :=
  match coerceToNumber value, coerceToNumber digits with
  | .ok lhs, .ok rhs => .ok (roundKernel lhs (truncateDigitsTowardZero rhs))
  | .error e, _ => .error e
  | _, .error e => .error e

theorem evalRoundPrepared_half_away_from_zero_positive :
    evalRoundPrepared (.number (125 / 100 : Rat)) (.number 1) = .ok (13 / 10 : Rat) := by
  native_decide

theorem evalRoundPrepared_half_away_from_zero_negative :
    evalRoundPrepared (.number (-125 / 100 : Rat)) (.number 1) = .ok (-13 / 10 : Rat) := by
  native_decide

theorem evalRoundPrepared_negative_digits :
    evalRoundPrepared (.number 123) (.number (-1)) = .ok 120 := by
  native_decide

theorem evalRoundPrepared_truncates_num_digits_toward_zero :
    evalRoundPrepared (.number (3 / 2 : Rat)) (.number (9 / 10 : Rat)) = .ok 2 := by
  native_decide

theorem roundMeta_profiles :
    roundMeta.kernelSignatureClass = KernelSignatureClass.numsToNum
    ∧ roundMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [roundMeta]

end OxFunc.Functions
