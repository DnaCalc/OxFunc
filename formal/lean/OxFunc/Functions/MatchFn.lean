import OxFunc.FunctionCore
import OxFunc.CoercionPrimitives
import OxFunc.Functions.Xmatch

namespace OxFunc.Functions

open OxFunc

def matchMeta : FunctionMeta := {
  functionId := "FUNC.MATCH"
  arity := { min := 2, max := 3 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.lookupMatchProfile
  kernelSignatureClass := KernelSignatureClass.lookupMatch
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

inductive MatchError where
  | coercion (e : CoercionError)
  | invalidMatchType (n : Rat)
  | notAvailable
  | emptyLookupArray
  deriving DecidableEq, Repr, Inhabited

inductive MatchApproximateMode where
  | ascendingNextSmaller
  | descendingNextLarger
  deriving DecidableEq, Repr, Inhabited

def mapXmatchError : XmatchError → MatchError
  | .coercion e => .coercion e
  | .invalidMatchMode n => .invalidMatchType n
  | .emptyLookupArray => .emptyLookupArray
  | .notAvailable => .notAvailable
  | .unsupportedMatchModeForSeed _ => .invalidMatchType 9
  | .invalidSearchMode _ | .unsupportedSearchModeForSeed _ => .coercion (.unsupportedKind "match_search_mode")

def containsWildcardSyntax : String → Bool
  | s =>
      let rec go : List Char → Bool
        | [] => false
        | '*' :: _ => true
        | '?' :: _ => true
        | '~' :: '*' :: _ => true
        | '~' :: '?' :: _ => true
        | '~' :: '~' :: _ => true
        | _ :: cs => go cs
      go s.toList

def toStrictComparable (v : CoercionInput) : Except MatchError XmatchComparable :=
  match toLookupComparable v with
  | Except.ok c => Except.ok c
  | Except.error (.coercion .missingArg) => Except.error .notAvailable
  | Except.error (.coercion .emptyCell) => Except.error .notAvailable
  | Except.error e => Except.error (mapXmatchError e)

def collectMatchCandidates (lookupArray : List CoercionInput) : Except MatchError (List XmatchComparable) := do
  let rec go (rest : List CoercionInput) (acc : List XmatchComparable) : Except MatchError (List XmatchComparable) :=
    match rest with
    | [] => Except.ok acc.reverse
    | x :: xs =>
        match toLookupCandidate x with
        | Except.error e => Except.error (mapXmatchError e)
        | Except.ok .skip => Except.error .notAvailable
        | Except.ok .blankCell => Except.error .notAvailable
        | Except.ok (.comparable c) => go xs (c :: acc)
  go lookupArray []

def firstGreaterAscendingList (candidates : List XmatchComparable) (needle : XmatchComparable) : Nat :=
  let rec go (fuel low high best : Nat) : Nat :=
    if fuel = 0 then best
    else if low ≤ high && high > 0 then
      let mid := low + ((high - low) / 2)
      let midIdx := mid - 1
      match comparableOrder (listGetD candidates midIdx) needle with
      | some .gt =>
          if mid = 1 then mid else go (fuel - 1) low (mid - 1) mid
      | some .eq | some .lt => go (fuel - 1) (mid + 1) high best
      | none => 0
    else best
  go (candidates.length + 1) 1 candidates.length 0

def evalMatchApproximate
    (lookupValue : CoercionInput)
    (lookupArray : List CoercionInput)
    (mode : MatchApproximateMode) : Except MatchError Nat := do
  let needle ← toStrictComparable lookupValue
  let candidates ← collectMatchCandidates lookupArray
  if candidates.isEmpty then
    Except.error .notAvailable
  else
    match mode with
    | .ascendingNextSmaller =>
        let firstGreater := firstGreaterAscendingList candidates needle
        if firstGreater = 0 then Except.ok candidates.length
        else if firstGreater = 1 then Except.error .notAvailable
        else Except.ok (firstGreater - 1)
    | .descendingNextLarger =>
        let firstLE := firstLessOrEqualDescending candidates needle
        if firstLE = 0 then
          Except.ok candidates.length
        else if comparableEq (listGetD candidates (firstLE - 1)) needle then
          Except.ok firstLE
        else if firstLE = 1 then
          Except.error .notAvailable
        else
          Except.ok (firstLE - 1)

def evalMatchPosition
    (lookupValue : CoercionInput)
    (lookupArray : List CoercionInput)
    (matchType : Option CoercionInput := none) : Except MatchError Nat := do
  match matchType with
  | none | some .missingArg =>
      evalMatchApproximate lookupValue lookupArray .ascendingNextSmaller
  | some v =>
      match coerceToNumber v with
      | Except.error e => Except.error (.coercion e)
      | Except.ok 1 => evalMatchApproximate lookupValue lookupArray .ascendingNextSmaller
      | Except.ok (-1) => evalMatchApproximate lookupValue lookupArray .descendingNextLarger
      | Except.ok 0 =>
          let xmatchMode :=
            match lookupValue with
            | .text s =>
                if containsWildcardSyntax s then some (.number 2) else some (.number 0)
            | _ => some (.number 0)
          match evalXmatchPositionWithBlankBehavior .notAvailable lookupValue lookupArray xmatchMode none with
          | Except.ok idx => Except.ok idx
          | Except.error e => Except.error (mapXmatchError e)
      | Except.ok n => Except.error (.invalidMatchType n)

def evalMatchExact
    (lookupValue : CoercionInput)
    (lookupArray : List CoercionInput) : Except MatchError Nat :=
  evalMatchPosition lookupValue lookupArray (some (.number 0))

theorem evalMatchExact_found :
    evalMatchExact (.number 3) [.number 1, .number 3] = Except.ok 2 := by
  native_decide

theorem evalMatchExact_not_found :
    evalMatchExact (.number 9) [.number 1, .number 3] = Except.error .notAvailable := by
  native_decide

theorem matchMeta_profiles :
    matchMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ matchMeta.coercionLiftProfile = CoercionLiftProfile.lookupMatchProfile
    ∧ matchMeta.kernelSignatureClass = KernelSignatureClass.lookupMatch
    ∧ matchMeta.fecDependencyProfile = FecDependencyProfile.refOnly
    ∧ matchMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [matchMeta]

theorem evalMatchWildcardEscape_exact :
    evalMatchPosition (.text "a~*") [.text "abc", .text "a*"] (some (.number 0)) = Except.ok 2 := by
  native_decide

theorem evalMatchBlankLookup_notAvailable :
    evalMatchPosition .emptyCell [.emptyCell, .number 1] (some (.number 0)) =
      Except.error .notAvailable := by
  native_decide

theorem evalMatchApproximateDuplicates :
    evalMatchPosition (.number 2) [.number 1, .number 2, .number 2, .number 2, .number 3]
      (some (.number 1)) = Except.ok 4 := by
  native_decide

theorem evalMatchApproximateUnsortedEmpirical :
    evalMatchPosition (.number (5 / 2 : Rat)) [.number 1, .number 3, .number 2, .number 4]
      (some (.number 1)) = Except.ok 1 := by
  native_decide

end OxFunc.Functions
