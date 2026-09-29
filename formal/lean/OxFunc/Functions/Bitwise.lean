import OxFunc.CoercionPrimitives

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptBitwise [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

/-- Exact integer substrate for the bitwise kernels. Rat models the finite
numeric input after scalar coercion; binary64 ingress is a separate seam.
Aligned with `bit_common.rs` and the build 20430/CV2 Value2 witnesses. -/
def bitOperandMax : Nat := 281474976710655

/-- Family-specific preparation before ordinary scalar coercion. Explicitly
omitted argument positions are zero; absent argument positions still fail arity. -/
def prepareBitwiseMissing (input : CoercionInput) : CoercionInput :=
  match input with
  | .missingArg => .number 0
  | other => other

theorem prepareBitwiseMissing_coerces_to_zero :
    coerceToNumber (prepareBitwiseMissing .missingArg) = .ok 0 := by
  rfl

def coerceBitOperandModel (n : Rat) : Except WorksheetErrorCode Nat :=
  if n < 0 ∨ n > (bitOperandMax : Rat) ∨ n.den ≠ 1 then .error .num
  else .ok n.num.toNat

def coerceBitShiftModel (n : Rat) : Except WorksheetErrorCode Int :=
  let truncated := n.num.tdiv (Int.ofNat n.den)
  if truncated.natAbs > 53 then .error .num else .ok truncated

inductive BitBinaryOperation where
  | andOp | orOp | xorOp
  deriving DecidableEq, Repr

def bitBinaryKernelModel (op : BitBinaryOperation) (lhs rhs : Rat) : Except WorksheetErrorCode Nat := do
  let lhs ← coerceBitOperandModel lhs
  let rhs ← coerceBitOperandModel rhs
  return match op with
    | .andOp => Nat.land lhs rhs
    | .orOp => Nat.lor lhs rhs
    | .xorOp => Nat.xor lhs rhs

/-- Empirical order: operand admission, zero shortcut, truncated count admission,
49..53 zero publication, direction and checked 48-bit result. In particular the
49..53 branch differs from the ordinary mathematical left-shift operation. -/
def bitShiftKernelModel (number shift : Rat) (left : Bool) : Except WorksheetErrorCode Nat := do
  let number ← coerceBitOperandModel number
  if number = 0 then return 0
  let count ← coerceBitShiftModel shift
  if count.natAbs > 48 then return 0
  let count := if left then count else -count
  let result := if count ≥ 0 then Nat.shiftLeft number count.toNat
                else Nat.shiftRight number (-count).toNat
  if result > bitOperandMax then .error .num else .ok result

theorem coerceBitOperandModel_fractional_rejected (n : Rat) (h : n.den ≠ 1) :
    coerceBitOperandModel n = .error .num := by
  simp [coerceBitOperandModel, h]

theorem bitShiftKernelModel_zero_shortcut (shift : Rat) (left : Bool) :
    bitShiftKernelModel 0 shift left = .ok 0 := by
  have h : coerceBitOperandModel 0 = .ok 0 := by native_decide
  unfold bitShiftKernelModel
  rw [h]
  rfl

theorem bitwiseModel_fractional_admission_examples :
    bitBinaryKernelModel .andOp (9 / 10) 1 = .error .num ∧
    bitBinaryKernelModel .orOp 1 (3 / 2) = .error .num ∧
    bitBinaryKernelModel .xorOp (3 / 2) 1 = .error .num := by
  native_decide

theorem bitwiseModel_shift_witnesses :
    bitShiftKernelModel 1 47 true = .ok 140737488355328 ∧
    bitShiftKernelModel 1 48 true = .error .num ∧
    bitShiftKernelModel 1 53 true = .ok 0 ∧
    bitShiftKernelModel 1 (-48) false = .error .num ∧
    bitShiftKernelModel 1 (-53) false = .ok 0 ∧
    bitShiftKernelModel 197029275778236 (-27 / 2) true = .ok 24051425265 := by
  native_decide

end OxFunc.Functions
