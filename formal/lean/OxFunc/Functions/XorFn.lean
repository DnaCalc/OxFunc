import OxFunc.FunctionCore
import OxFunc.Functions.AndFn

namespace OxFunc.Functions

open OxFunc

def xorMeta : FunctionMeta := {
  functionId := "FUNC.XOR"
  arity := { min := 1, max := 255 }
  determinism := .deterministic
  volatility := .nonvolatile
  hostInteraction := .none
  threadSafety := .safePure
  argPreparationProfile := .valuesOnlyPreAdapter
  coercionLiftProfile := .custom
  kernelSignatureClass := .custom
  fecDependencyProfile := .none
  surfaceFecDependencyProfile := .refOnly
}

/-- XOR uses the same prepared-item policy as AND/OR and folds odd parity over every
contributing logical value. This is the value-only substrate, after reference expansion. -/
def evalXorPrepared (args : List AndPreparedArg) : Except WorksheetErrorCode Bool :=
  let rec loop : List AndPreparedArg → Bool → Bool → Except WorksheetErrorCode Bool
    | [], sawValue, parity => if sawValue then .ok parity else .error .value
    | x :: xs, sawValue, parity =>
        match andArgumentTruth x with
        | .error code => .error code
        | .ok none => loop xs sawValue parity
        | .ok (some truth) => loop xs true (if truth then !parity else parity)
  loop args false false

private instance instDecidableEqExceptXor [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

theorem evalXorPrepared_direct_text_and_parity_rule :
    evalXorPrepared [⟨.directScalar, .text "TrUe"⟩] = .ok true
    ∧ evalXorPrepared [⟨.directScalar, .text "FALSE"⟩] = .ok false
    ∧ evalXorPrepared [⟨.directScalar, .text "TRUE"⟩, ⟨.directScalar, .text "true"⟩] = .ok false
    ∧ evalXorPrepared [⟨.directScalar, .logical true⟩, ⟨.directScalar, .text "x"⟩,
        ⟨.directScalar, .text "1"⟩, ⟨.arrayLike, .text "TRUE"⟩] = .ok true
    ∧ evalXorPrepared [⟨.directScalar, .text "1"⟩, ⟨.directScalar, .text " TRUE "⟩] = .error .value := by
  native_decide

theorem evalXorPrepared_direct_text_does_not_mask_first_error :
    evalXorPrepared [⟨.directScalar, .text "x"⟩, ⟨.directScalar, .error .div0⟩] = .error .div0
    ∧ evalXorPrepared [⟨.directScalar, .text "TRUE"⟩, ⟨.directScalar, .text "x"⟩,
        ⟨.directScalar, .error .na⟩, ⟨.directScalar, .error .div0⟩] = .error .na := by
  native_decide

theorem xorMeta_profiles :
    xorMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ xorMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ xorMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [xorMeta]

end OxFunc.Functions
