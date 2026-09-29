import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptMround [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def mroundMeta : FunctionMeta := {
  functionId := "FUNC.MROUND"
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

def evalMroundSurfaceClass (x y : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber x, coerceToNumber y with
  | .ok _, .ok 0 => .ok "number"
  | .ok n, .ok m =>
      if (0 ≤ n ∧ 0 ≤ m) ∨ (n ≤ 0 ∧ m ≤ 0) then .ok "number" else .error .num
  | .error (.worksheetError code), _ => .error code
  | _, .error (.worksheetError code) => .error code
  | .error _, _ => .error .value
  | _, .error _ => .error .value

theorem evalMround_sign_mismatch_is_num :
    evalMroundSurfaceClass (.number 10) (.number (-3)) = .error .num := by
  native_decide

theorem mroundMeta_profiles :
    mroundMeta.kernelSignatureClass = KernelSignatureClass.numsToNum
    ∧ mroundMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [mroundMeta]

/-- W111 observed arithmetic graph. The supplied division and multiplication
bind to RN64 followed by binary64 publication on the captured x86_64 host.
The fractional cutoff is bracketed by adjacent public Excel observations;
its internal derivation is not asserted. This is not a whole-function proof. -/
def mroundWithPrimitives (divide multiply : Float → Float → Float)
    (number multiple : Float) : Except WorksheetErrorCode Float :=
  if multiple == 0 || number == 0 then .ok 0
  else if (number < 0) != (multiple < 0) then .error .num
  else
    let quotient := divide number.abs multiple.abs
    if !quotient.isFinite then .error .num
    else
      let lower := quotient.floor
      let count := lower + (if quotient - lower ≥ Float.ofBits 0x3fdfffffffffffa6 then 1 else 0)
      let result := multiply count multiple
      if !result.isFinite then .error .num
      else .ok (if result == 0 then 0 else result)

theorem mround_fraction_boundary_binding :
    (mroundWithPrimitives (· / ·) (· * ·) (Float.ofBits 0x3fdfffffffffffa5) 1).toOption.map Float.toBits = some 0 ∧
    (mroundWithPrimitives (· / ·) (· * ·) (Float.ofBits 0x3fdfffffffffffa6) 1).toOption.map Float.toBits = some 0x3ff0000000000000 := by
  native_decide

theorem mround_negative_result_and_zero_binding :
    (mroundWithPrimitives (· / ·) (· * ·) (-10) (-3)).toOption.map Float.toBits = some 0xc022000000000000 ∧
    (mroundWithPrimitives (· / ·) (· * ·) (-0.1) (-1)).toOption.map Float.toBits = some 0 := by
  native_decide

theorem mround_overflow_binding :
    (mroundWithPrimitives (fun _ _ => Float.ofBits 0x7ff0000000000000)
      (· * ·) 1 1).toOption = none ∧
    (mroundWithPrimitives (fun _ _ => 1)
      (fun _ _ => Float.ofBits 0x7ff0000000000000) 1 1).toOption = none := by
  native_decide

end OxFunc.Functions
