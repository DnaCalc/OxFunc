import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptPower [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def powerMeta : FunctionMeta := {
  functionId := "FUNC.POWER"
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

def powerNatPublication : Rat → Nat → Rat
  | _, 0 => 1
  | base, n + 1 => powerNatPublication base n * base

def powerIntPublication (base : Rat) : Int → Rat
  | .ofNat n => powerNatPublication base n
  | .negSucc n => 1 / powerNatPublication base (n + 1)

def usesIntegerPublicationLane (power : Rat) : Bool :=
  power.den = 1

def evalPowerSurfaceClass (x y : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber x, coerceToNumber y with
  | .ok 0, .ok p =>
      if p = 0 then .error .num else if p < 0 then .error .div0 else .ok "number"
  | .ok n, .ok p =>
      if n < 0 ∧ p.den ≠ 1 then .error .num else .ok "number"
  | .error (.worksheetError code), _ => .error code
  | _, .error (.worksheetError code) => .error code
  | .error _, _ => .error .value
  | _, .error _ => .error .value

theorem evalPower_zero_negative_power_is_div0 :
    evalPowerSurfaceClass (.number 0) (.number (-1)) = .error .div0 := by
  native_decide

theorem evalPower_zero_zero_is_num :
    evalPowerSurfaceClass (.number 0) (.number 0) = .error .num := by
  native_decide

theorem power_integer_publication_seed_105_10 :
    powerNatPublication (21 / 20 : Rat) 10 = (16679880978201 / 10240000000000 : Rat) := by
  native_decide

theorem power_integer_publication_seed_151_150_10 :
    powerNatPublication (151 / 150 : Rat) 10 =
      (6162677950336718514001 / 5766503906250000000000 : Rat) := by
  native_decide

theorem power_integer_publication_negative_seed :
    powerIntPublication (2 : Rat) (-3) = (1 / 8 : Rat) := by
  native_decide

theorem powerMeta_profiles :
    powerMeta.kernelSignatureClass = KernelSignatureClass.numsToNum
    ∧ powerMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [powerMeta]

end OxFunc.Functions
