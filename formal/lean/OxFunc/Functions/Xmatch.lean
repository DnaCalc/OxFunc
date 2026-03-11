import OxFunc.FunctionCore
import OxFunc.CoercionPrimitives

namespace OxFunc.Functions

open OxFunc

instance instDecidableEqExcept [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def xmatchMeta : FunctionMeta := {
  functionId := "FUNC.XMATCH"
  arity := { min := 2, max := 4 }
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

inductive XmatchMatchMode where
  | exact
  | exactOrNextLarger
  | exactOrNextSmaller
  | wildcard
  deriving DecidableEq, Repr, Inhabited

inductive XmatchSearchMode where
  | firstToLast
  | lastToFirst
  | binaryAscending
  | binaryDescending
  deriving DecidableEq, Repr, Inhabited

inductive BlankLookupBehavior where
  | matchBlankCells
  | notAvailable
  deriving DecidableEq, Repr, Inhabited

inductive XmatchError where
  | emptyLookupArray
  | coercion (e : CoercionError)
  | invalidMatchMode (n : Rat)
  | invalidSearchMode (n : Rat)
  | unsupportedMatchModeForSeed (m : XmatchMatchMode)
  | unsupportedSearchModeForSeed (m : XmatchSearchMode)
  | notAvailable
  deriving DecidableEq, Repr, Inhabited

inductive XmatchComparable where
  | number (n : Rat)
  | text (s : String)
  | logical (b : Bool)
  deriving DecidableEq, Repr, Inhabited

inductive LookupNeedle where
  | comparable (v : XmatchComparable)
  | blankCell
  deriving DecidableEq, Repr, Inhabited

inductive LookupCandidate where
  | comparable (v : XmatchComparable)
  | blankCell
  | skip
  deriving DecidableEq, Repr, Inhabited

inductive WildcardAtom where
  | anySeq
  | anyChar
  | literal (c : Char)
  deriving DecidableEq, Repr, Inhabited

def parseMatchMode (n : Rat) : Except XmatchError XmatchMatchMode :=
  if n = 0 then Except.ok .exact
  else if n = 1 then Except.ok .exactOrNextLarger
  else if n = -1 then Except.ok .exactOrNextSmaller
  else if n = 2 then Except.ok .wildcard
  else Except.error (.invalidMatchMode n)

def parseSearchMode (n : Rat) : Except XmatchError XmatchSearchMode :=
  if n = 1 then Except.ok .firstToLast
  else if n = -1 then Except.ok .lastToFirst
  else if n = 2 then Except.ok .binaryAscending
  else if n = -2 then Except.ok .binaryDescending
  else Except.error (.invalidSearchMode n)

def parseOptionalMatchMode : Option CoercionInput → Except XmatchError XmatchMatchMode
  | none => Except.ok .exact
  | some .missingArg => Except.ok .exact
  | some v =>
      match coerceToNumber v with
      | Except.ok n => parseMatchMode n
      | Except.error e => Except.error (.coercion e)

def parseOptionalSearchMode : Option CoercionInput → Except XmatchError XmatchSearchMode
  | none => Except.ok .firstToLast
  | some .missingArg => Except.ok .firstToLast
  | some v =>
      match coerceToNumber v with
      | Except.ok n => parseSearchMode n
      | Except.error e => Except.error (.coercion e)

def comparableTextEq (a b : String) : Bool :=
  a.toLower = b.toLower

def comparableEq (lhs rhs : XmatchComparable) : Bool :=
  match lhs, rhs with
  | .number a, .number b => a = b
  | .text a, .text b => comparableTextEq a b
  | .logical a, .logical b => a = b
  | _, _ => false

def comparableOrder : XmatchComparable → XmatchComparable → Option Ordering
  | .number a, .number b =>
      if a < b then some .lt else if a = b then some .eq else some .gt
  | .text a, .text b => some (compare a.toLower b.toLower)
  | .logical false, .logical false => some .eq
  | .logical false, .logical true => some .lt
  | .logical true, .logical false => some .gt
  | .logical true, .logical true => some .eq
  | _, _ => none

def toLookupComparable : CoercionInput → Except XmatchError XmatchComparable
  | .number n => Except.ok (.number n)
  | .text s => Except.ok (.text s)
  | .logical b => Except.ok (.logical b)
  | .error code => Except.error (.coercion (.worksheetError code))
  | .missingArg => Except.error (.coercion .missingArg)
  | .emptyCell => Except.error (.coercion .emptyCell)

def toLookupNeedle (blankBehavior : BlankLookupBehavior) : CoercionInput → Except XmatchError LookupNeedle
  | .missingArg | .emptyCell =>
      match blankBehavior with
      | .matchBlankCells => Except.ok .blankCell
      | .notAvailable => Except.error .notAvailable
  | v =>
      match toLookupComparable v with
      | Except.ok c => Except.ok (.comparable c)
      | Except.error e => Except.error e

def toLookupCandidate : CoercionInput → Except XmatchError LookupCandidate
  | .emptyCell => Except.ok .blankCell
  | .missingArg => Except.ok .skip
  | .error _ => Except.ok .skip
  | v =>
      match toLookupComparable v with
      | Except.ok c => Except.ok (.comparable c)
      | Except.error e => Except.error e

def parseWildcardAtoms : List Char → List WildcardAtom
  | [] => []
  | '~' :: c :: cs => .literal c :: parseWildcardAtoms cs
  | '*' :: cs => .anySeq :: parseWildcardAtoms cs
  | '?' :: cs => .anyChar :: parseWildcardAtoms cs
  | c :: cs => .literal c :: parseWildcardAtoms cs

def wildcardMatchFuel : Nat → List WildcardAtom → List Char → Bool
  | 0, _, _ => false
  | _ + 1, [], [] => true
  | _ + 1, [], _ => false
  | fuel + 1, .anySeq :: ps, cs =>
      wildcardMatchFuel fuel ps cs ||
        match cs with
        | [] => false
        | _ :: rest => wildcardMatchFuel fuel (.anySeq :: ps) rest
  | fuel + 1, .anyChar :: ps, _ :: cs => wildcardMatchFuel fuel ps cs
  | _ + 1, .anyChar :: _, [] => false
  | fuel + 1, .literal c :: ps, d :: cs =>
      if c = d then wildcardMatchFuel fuel ps cs else false
  | _ + 1, .literal _ :: _, [] => false

def wildcardMatch (pattern text : String) : Bool :=
  let atoms := parseWildcardAtoms pattern.toLower.toList
  let chars := text.toLower.toList
  wildcardMatchFuel ((atoms.length + 1) * (chars.length + 1)) atoms chars

def candidateMatches (needle : LookupNeedle) (candidate : LookupCandidate) (matchMode : XmatchMatchMode) : Bool :=
  match needle, candidate, matchMode with
  | .blankCell, .blankCell, .exact => true
  | .comparable lhs, .comparable rhs, .wildcard =>
      match lhs, rhs with
      | .text pattern, .text text => wildcardMatch pattern text
      | _, _ => false
  | .comparable lhs, .comparable rhs, _ => comparableEq lhs rhs
  | _, _, _ => false

def enumerateFrom (idx : Nat) : List α → List (Nat × α)
  | [] => []
  | x :: xs => (idx, x) :: enumerateFrom (idx + 1) xs

def prepareLookupPairs (lookupArray : List CoercionInput) :
    Except XmatchError (List (Nat × LookupCandidate)) := do
  let candidates ← lookupArray.mapM toLookupCandidate
  Except.ok (enumerateFrom 1 candidates)

def orderedPairs (searchMode : XmatchSearchMode) (lookupArray : List CoercionInput) :
    Except XmatchError (List (Nat × LookupCandidate)) := do
  let pairs ← prepareLookupPairs lookupArray
  let ordered :=
    match searchMode with
    | .firstToLast | .binaryAscending => pairs
    | .lastToFirst | .binaryDescending => pairs.reverse
  Except.ok ordered

def chooseApproxBetter
    (matchMode : XmatchMatchMode)
    (current : Option (Nat × XmatchComparable))
    (candidate : Nat × XmatchComparable) : Option (Nat × XmatchComparable) :=
  match matchMode, current with
  | .exact, _ => current
  | .wildcard, _ => current
  | .exactOrNextLarger, none => some candidate
  | .exactOrNextSmaller, none => some candidate
  | .exactOrNextLarger, some (_, best) =>
      match comparableOrder candidate.2 best with
      | some .lt => some candidate
      | _ => current
  | .exactOrNextSmaller, some (_, best) =>
      match comparableOrder candidate.2 best with
      | some .gt => some candidate
      | _ => current

def scanExactOrApprox
    (needle : LookupNeedle)
    (matchMode : XmatchMatchMode)
    (searchMode : XmatchSearchMode)
    (lookupArray : List CoercionInput) : Except XmatchError Nat := do
  let pairs ← orderedPairs searchMode lookupArray
  let rec go (rest : List (Nat × LookupCandidate)) (best : Option (Nat × XmatchComparable)) :
      Except XmatchError Nat :=
    match rest with
    | [] =>
        match best with
        | some (idx, _) => Except.ok idx
        | none => Except.error .notAvailable
    | (idx, candidate) :: tail =>
        if candidateMatches needle candidate matchMode then
          Except.ok idx
        else
          match needle, candidate, matchMode with
          | .comparable lookupValue, .comparable cand, .exactOrNextLarger =>
              match comparableOrder cand lookupValue with
              | some .gt => go tail (chooseApproxBetter matchMode best (idx, cand))
              | _ => go tail best
          | .comparable lookupValue, .comparable cand, .exactOrNextSmaller =>
              match comparableOrder cand lookupValue with
              | some .lt => go tail (chooseApproxBetter matchMode best (idx, cand))
              | _ => go tail best
          | _, _, _ => go tail best
  go pairs none

def collectBinaryComparables (lookupArray : List CoercionInput) :
    Except XmatchError (Option (List XmatchComparable)) := do
  let rec go (rest : List CoercionInput) (acc : List XmatchComparable) :
      Except XmatchError (Option (List XmatchComparable)) :=
    match rest with
    | [] => Except.ok (some acc.reverse)
    | x :: xs =>
        match toLookupCandidate x with
        | Except.error e => Except.error e
        | Except.ok .skip => Except.ok none
        | Except.ok .blankCell => Except.ok none
        | Except.ok (.comparable c) => go xs (c :: acc)
  go lookupArray []

def listGetD [Inhabited α] : List α → Nat → α
  | [], _ => default
  | x :: _, 0 => x
  | _ :: xs, n + 1 => listGetD xs n

def lowerBoundAscending (candidates : List XmatchComparable) (needle : XmatchComparable) : Nat :=
  let rec go (fuel low high : Nat) : Nat :=
    if fuel = 0 then low
    else if low < high then
      let mid := low + ((high - low) / 2)
      match comparableOrder (listGetD candidates mid) needle with
      | some .lt => go (fuel - 1) (mid + 1) high
      | _ => go (fuel - 1) low mid
    else low
  go (candidates.length + 1) 0 candidates.length

def firstLessDescending (candidates : List XmatchComparable) (needle : XmatchComparable) : Nat :=
  let rec go (fuel low high : Nat) : Nat :=
    if fuel = 0 then low
    else if low < high then
      let mid := low + ((high - low) / 2)
      match comparableOrder (listGetD candidates mid) needle with
      | some .lt => go (fuel - 1) low mid
      | _ => go (fuel - 1) (mid + 1) high
    else low
  go (candidates.length + 1) 0 candidates.length

def firstLessOrEqualDescending (candidates : List XmatchComparable) (needle : XmatchComparable) : Nat :=
  let rec go (fuel low high : Nat) : Nat :=
    if fuel = 0 then low
    else if low < high then
      let mid := low + ((high - low) / 2)
      match comparableOrder (listGetD candidates mid) needle with
      | some .gt => go (fuel - 1) (mid + 1) high
      | some .eq => go (fuel - 1) low mid
      | some .lt => go (fuel - 1) low mid
      | none => candidates.length
    else low
  go (candidates.length + 1) 0 candidates.length

def xmatchBinarySearch
    (needle : XmatchComparable)
    (matchMode : XmatchMatchMode)
    (searchMode : XmatchSearchMode)
    (lookupArray : List CoercionInput) : Except XmatchError Nat := do
  let some candidates ← collectBinaryComparables lookupArray
    | Except.error .notAvailable
  if candidates.isEmpty then
    Except.error .notAvailable
  else
    match searchMode with
    | .binaryAscending =>
        let lower := lowerBoundAscending candidates needle
        if _h : lower < candidates.length then
          if comparableEq (listGetD candidates lower) needle then
            Except.ok (lower + 1)
          else
            match matchMode with
              | .exact => Except.error .notAvailable
              | .exactOrNextLarger => Except.ok (lower + 1)
              | .exactOrNextSmaller =>
                if lower = 0 then Except.error .notAvailable else Except.ok lower
              | .wildcard => Except.error (.coercion (.unsupportedKind "wildcard_binary_search"))
        else
          match matchMode with
            | .exact => Except.error .notAvailable
            | .exactOrNextLarger => Except.error .notAvailable
            | .exactOrNextSmaller => Except.ok candidates.length
            | .wildcard => Except.error (.coercion (.unsupportedKind "wildcard_binary_search"))
    | .binaryDescending =>
        let lower := firstLessDescending candidates needle
        if lower > 0 then
          let exactIdx := lower - 1
          if _h : exactIdx < candidates.length then
            if comparableEq (listGetD candidates exactIdx) needle then
              Except.ok lower
            else
              match matchMode with
              | .exact => Except.error .notAvailable
              | .exactOrNextLarger =>
                  if lower = 1 then Except.error .notAvailable else Except.ok (lower - 1)
              | .exactOrNextSmaller =>
                  if _h₂ : lower < candidates.length then Except.ok (lower + 1) else Except.error .notAvailable
              | .wildcard => Except.error (.coercion (.unsupportedKind "wildcard_binary_search"))
          else
            Except.error .notAvailable
        else
          let firstLE := firstLessOrEqualDescending candidates needle
          match matchMode with
          | .exact =>
              if _h : firstLE < candidates.length ∧ comparableEq (listGetD candidates firstLE) needle then
                Except.ok (firstLE + 1)
              else Except.error .notAvailable
          | .exactOrNextLarger => Except.ok candidates.length
          | .exactOrNextSmaller =>
              if firstLE < candidates.length then Except.ok (firstLE + 1) else Except.error .notAvailable
          | .wildcard => Except.error (.coercion (.unsupportedKind "wildcard_binary_search"))
    | .firstToLast | .lastToFirst => Except.error (.unsupportedSearchModeForSeed searchMode)

def evalXmatchPositionWithBlankBehavior
    (blankBehavior : BlankLookupBehavior)
    (lookupValue : CoercionInput)
    (lookupArray : List CoercionInput)
    (matchModeArg : Option CoercionInput := none)
    (searchModeArg : Option CoercionInput := none) : Except XmatchError Nat := do
  if lookupArray.isEmpty then
    Except.error .emptyLookupArray
  else
    let matchMode ← parseOptionalMatchMode matchModeArg
    let searchMode ← parseOptionalSearchMode searchModeArg
    match matchMode, searchMode with
    | .wildcard, .binaryAscending | .wildcard, .binaryDescending =>
        Except.error (.coercion (.unsupportedKind "wildcard_binary_search"))
    | _, _ =>
        let needle ← toLookupNeedle blankBehavior lookupValue
        match needle with
        | .blankCell =>
            match matchMode, searchMode with
            | .exact, .firstToLast | .exact, .lastToFirst =>
                scanExactOrApprox needle matchMode searchMode lookupArray
            | .exact, .binaryAscending | .exact, .binaryDescending =>
                Except.error .notAvailable
            | _, _ => Except.error .notAvailable
        | .comparable comparable =>
            match searchMode with
            | .binaryAscending | .binaryDescending =>
                xmatchBinarySearch comparable matchMode searchMode lookupArray
            | .firstToLast | .lastToFirst =>
                scanExactOrApprox needle matchMode searchMode lookupArray

def evalXmatchPosition
    (lookupValue : CoercionInput)
    (lookupArray : List CoercionInput)
    (matchModeArg : Option CoercionInput := none)
    (searchModeArg : Option CoercionInput := none) : Except XmatchError Nat :=
  evalXmatchPositionWithBlankBehavior .matchBlankCells lookupValue lookupArray matchModeArg searchModeArg

def evalXmatchAdapter
    (lookupValue : CoercionInput)
    (lookupArray : List CoercionInput)
    (matchModeArg : Option CoercionInput := none)
    (searchModeArg : Option CoercionInput := none) :
    Except XmatchError Value := do
  let idx ← evalXmatchPosition lookupValue lookupArray matchModeArg searchModeArg
  Except.ok (Value.number idx)

theorem xmatchMeta_seed_profiles :
    xmatchMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ xmatchMeta.coercionLiftProfile = CoercionLiftProfile.lookupMatchProfile
    ∧ xmatchMeta.kernelSignatureClass = KernelSignatureClass.lookupMatch
    ∧ xmatchMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ xmatchMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [xmatchMeta]

theorem evalXmatch_defaults_exact_forward :
    parseOptionalMatchMode none = Except.ok .exact
    ∧ parseOptionalSearchMode none = Except.ok .firstToLast := by
  simp [parseOptionalMatchMode, parseOptionalSearchMode]

theorem evalXmatch_reverse_returns_last_match :
    evalXmatchPosition (.number 2) [.number 2, .number 1, .number 2]
      (some (.number 0)) (some (.number (-1))) = Except.ok 3 := by
  native_decide

theorem evalXmatch_not_available_lane :
    evalXmatchPosition (.number 9) [.number 1, .number 2] = Except.error .notAvailable := by
  native_decide

theorem evalXmatch_lookup_array_error_skipped :
    evalXmatchPosition (.number 2) [.number 1, .error .div0, .number 2] = Except.ok 3 := by
  native_decide

theorem evalXmatch_blank_lookup_matches_true_blank :
    evalXmatchPosition .emptyCell [.emptyCell, .text "", .number 1] (some (.number 0)) =
      Except.ok 1 := by
  native_decide

theorem evalXmatch_empty_string_matches_formula_empty :
    evalXmatchPosition (.text "") [.emptyCell, .text "", .number 1] (some (.number 0)) =
      Except.ok 2 := by
  native_decide

theorem evalXmatch_binary_unsorted_larger_empirical :
    evalXmatchPosition (.number (5 / 2 : Rat)) [.number 3, .number 1, .number 4, .number 2]
      (some (.number 1)) (some (.number 2)) = Except.ok 3 := by
  native_decide

theorem evalXmatch_binary_descending_duplicate_empirical :
    evalXmatchPosition (.number 2) [.number 3, .number 2, .number 2, .number 2, .number 1]
      (some (.number 0)) (some (.number (-2))) = Except.ok 4 := by
  native_decide

theorem evalXmatch_deterministic
    (lookupValue : CoercionInput)
    (lookupArray : List CoercionInput)
    (matchModeArg searchModeArg : Option CoercionInput) :
    evalXmatchAdapter lookupValue lookupArray matchModeArg searchModeArg =
      evalXmatchAdapter lookupValue lookupArray matchModeArg searchModeArg := rfl

end OxFunc.Functions
