import OxFunc.FunctionCore
import OxFunc.CoercionPrimitives

namespace OxFunc.Functions

open OxFunc

def matchMeta : FunctionMeta := {
  functionId := "FUNC.MATCH"
  arity := { min := 2, max := 3 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.lookupMatchProfile
  kernelSignatureClass := KernelSignatureClass.lookupMatch
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

inductive MatchComparable where
  | number (n : Rat)
  | text (s : String)
  | logical (b : Bool)
  deriving DecidableEq, Repr

def toMatchComparable : CoercionInput → Except CoercionError MatchComparable
  | .number n => Except.ok (.number n)
  | .text s => Except.ok (.text s)
  | .logical b => Except.ok (.logical b)
  | .error code => Except.error (.worksheetError code)
  | .missingArg => Except.error .missingArg
  | .emptyCell => Except.error .emptyCell

def candidateComparable : CoercionInput → Option MatchComparable
  | .number n => some (.number n)
  | .text s => some (.text s)
  | .logical b => some (.logical b)
  | .error _ => none
  | .missingArg => none
  | .emptyCell => none

def matchExactIndex
    (needle : MatchComparable) : List CoercionInput → Nat → Except String (Option Nat)
  | [], _ => Except.ok none
  | x :: xs, idx =>
      match candidateComparable x with
      | none => matchExactIndex needle xs (idx + 1)
      | some c =>
          if c = needle then
            Except.ok (some idx)
          else
            matchExactIndex needle xs (idx + 1)

def evalMatchExact
    (lookupValue : CoercionInput)
    (lookupArray : List CoercionInput) : Except String Nat :=
  match toMatchComparable lookupValue with
  | Except.error _ => Except.error "coercion"
  | Except.ok needle =>
      match matchExactIndex needle lookupArray 1 with
      | Except.error e => Except.error e
      | Except.ok none => Except.error "na"
      | Except.ok (some idx) => Except.ok idx

theorem evalMatchExact_found :
    evalMatchExact (.number 3) [.number 1, .number 3] = Except.ok 2 := by
  simp [evalMatchExact, toMatchComparable, matchExactIndex, candidateComparable]

theorem evalMatchExact_not_found :
    evalMatchExact (.number 9) [.number 1, .number 3] = Except.error "na" := by
  simp [evalMatchExact, toMatchComparable, matchExactIndex, candidateComparable]

theorem matchMeta_profiles :
    matchMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ matchMeta.coercionLiftProfile = CoercionLiftProfile.lookupMatchProfile
    ∧ matchMeta.kernelSignatureClass = KernelSignatureClass.lookupMatch := by
  simp [matchMeta]

end OxFunc.Functions
