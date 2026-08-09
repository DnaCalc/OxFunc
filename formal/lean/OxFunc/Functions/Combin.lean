import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptCombin [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def combinMeta : FunctionMeta := {
  functionId := "FUNC.COMBIN"
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

def evalCombinSurfaceClass (x y : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber x, coerceToNumber y with
  | .ok n, .ok k =>
      if n < 0 ∨ k < 0 ∨ n < k then .error .num else .ok "number"
  | .error (.worksheetError code), _ => .error code
  | _, .error (.worksheetError code) => .error code
  | .error _, _ => .error .value
  | _, .error _ => .error .value

/-!
W109 executable route binding for the current-reference COMBIN numeric body.
Lean records the load-bearing order and store barriers without duplicating the
Rust x87 floating-point backend. After complement reduction, the factor loop
runs with ascending numerator `(n-k+1)..(n-1)` and denominator `2..k`; each
quotient and accumulator product is stored from x87 PC64 to binary64, and `n`
is multiplied only after that loop through the same stored-x87 operation.
-/
inductive CombinCyclicPublicationSite where
  | factorDivisionStore
  | accumulatorProductStore
  | finalNProductStore
  deriving DecidableEq, Repr

def combinCyclicPublicationSchedule : List CombinCyclicPublicationSite :=
  [.factorDivisionStore, .accumulatorProductStore, .finalNProductStore]

structure CombinPublicationRoute where
  complementReduction : Bool
  ascendingCyclicFactors : Bool
  denominatorStartsAtTwo : Bool
  quotientStoredX87 : Bool
  accumulatorStoredX87 : Bool
  nMultipliedLastStoredX87 : Bool
  deriving DecidableEq, Repr

def combinPublicationRoute : CombinPublicationRoute := {
  complementReduction := true
  ascendingCyclicFactors := true
  denominatorStartsAtTwo := true
  quotientStoredX87 := true
  accumulatorStoredX87 := true
  nMultipliedLastStoredX87 := true
}

theorem evalCombin_overflow_count_is_num :
    evalCombinSurfaceClass (.number 5) (.number 6) = .error .num := by
  native_decide

theorem combinMeta_profiles :
    combinMeta.kernelSignatureClass = KernelSignatureClass.numsToNum
    ∧ combinMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [combinMeta]

theorem combin_publication_route_is_cyclic_stored_x87 :
    combinCyclicPublicationSchedule =
      [.factorDivisionStore, .accumulatorProductStore, .finalNProductStore]
    ∧ combinPublicationRoute.complementReduction = true
    ∧ combinPublicationRoute.ascendingCyclicFactors = true
    ∧ combinPublicationRoute.denominatorStartsAtTwo = true
    ∧ combinPublicationRoute.quotientStoredX87 = true
    ∧ combinPublicationRoute.accumulatorStoredX87 = true
    ∧ combinPublicationRoute.nMultipliedLastStoredX87 = true := by
  native_decide

end OxFunc.Functions
