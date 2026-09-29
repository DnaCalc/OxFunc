import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptFact [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def factMeta : FunctionMeta := {
  functionId := "FUNC.FACT"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def evalFactSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok n => if n < 0 ∨ n ≥ 171 then .error .num else .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

/-- Executable binary64 reverse-product substrate. Integer admission bounds
the recursion to at most 170 factors; the runtime and this binding publish
after each multiplication in the same descending order. -/
def factReverseProductBinding : Nat → Float → Float
  | 0, acc => acc
  | n + 1, acc => factReverseProductBinding n (acc * (n + 1).toFloat)

def factFiniteBinding (input : Float) : Option UInt64 :=
  if !(input ≥ 0.0 && input < 171.0) then none
  else some ((factReverseProductBinding input.toUInt64.toNat 1.0).toBits)

theorem factFiniteBinding_live_bits :
    factFiniteBinding 25.0 = some 0x4529a940c33f6120 ∧
    factFiniteBinding 100.0 = some 0x60bb30964ec395de ∧
    factFiniteBinding 170.0 = some 0x7fa4ab786441863d ∧
    factFiniteBinding (-0.1) = none ∧
    factFiniteBinding 171.0 = none := by native_decide

theorem evalFact_negative_is_num :
    evalFactSurfaceClass (.number (-1)) = .error .num := by
  native_decide

-- Excel build 20430/CV2, Value2 probes retained in the W111 2026-09-29 campaign.
-- The sign check precedes truncation; positive fractions below 171 remain admitted.
theorem evalFact_negative_fraction_is_num :
    evalFactSurfaceClass (.number ((-1 : Rat) / 10)) = .error .num := by
  native_decide

theorem evalFact_positive_fraction_is_number :
    evalFactSurfaceClass (.number ((1709 : Rat) / 10)) = .ok "number" := by
  native_decide

theorem evalFact_overflow_is_num :
    evalFactSurfaceClass (.number 171) = .error .num := by
  native_decide

theorem factMeta_profiles :
    factMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ factMeta.coercionLiftProfile = CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
    ∧ factMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [factMeta]

end OxFunc.Functions
