import OxFunc.FunctionCore
import OxFunc.CoercionPrimitives

namespace OxFunc.Functions

open OxFunc

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
  deriving DecidableEq, Repr

inductive XmatchSearchMode where
  | firstToLast
  | lastToFirst
  | binaryAscending
  | binaryDescending
  deriving DecidableEq, Repr

inductive XmatchError where
  | emptyLookupArray
  | coercion (e : CoercionError)
  | invalidMatchMode (n : Rat)
  | invalidSearchMode (n : Rat)
  | unsupportedMatchModeForSeed (m : XmatchMatchMode)
  | unsupportedSearchModeForSeed (m : XmatchSearchMode)
  | notAvailable
  deriving DecidableEq, Repr

inductive XmatchComparable where
  | number (n : Rat)
  | text (s : String)
  | logical (b : Bool)
  deriving DecidableEq, Repr

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

def parseOptionalMatchMode (arg : Option CoercionInput) : Except XmatchError XmatchMatchMode :=
  match arg with
  | none => Except.ok .exact
  | some a =>
      match coerceToNumber a with
      | Except.ok n => parseMatchMode n
      | Except.error e => Except.error (.coercion e)

def parseOptionalSearchMode (arg : Option CoercionInput) : Except XmatchError XmatchSearchMode :=
  match arg with
  | none => Except.ok .firstToLast
  | some a =>
      match coerceToNumber a with
      | Except.ok n => parseSearchMode n
      | Except.error e => Except.error (.coercion e)

def toLookupValueComparable : CoercionInput → Except XmatchError XmatchComparable
  | .number n => Except.ok (.number n)
  | .text s => Except.ok (.text s)
  | .logical b => Except.ok (.logical b)
  | .error code => Except.error (.coercion (.worksheetError code))
  | .missingArg => Except.error (.coercion .missingArg)
  | .emptyCell => Except.error (.coercion .emptyCell)

def toLookupCandidateComparable :
    CoercionInput → Except XmatchError (Option XmatchComparable)
  | .number n => Except.ok (some (.number n))
  | .text s => Except.ok (some (.text s))
  | .logical b => Except.ok (some (.logical b))
  | .error _ => Except.ok none
  | .missingArg => Except.ok none
  | .emptyCell => Except.ok none

def comparableEq (lhs rhs : XmatchComparable) : Bool :=
  match lhs, rhs with
  | .number a, .number b => a = b
  | .text a, .text b => a = b
  | .logical a, .logical b => a = b
  | _, _ => false

def findForwardIndex
    (needle : XmatchComparable) : List CoercionInput → Nat → Except XmatchError (Option Nat)
  | [], _ => Except.ok none
  | x :: xs, idx =>
      match toLookupCandidateComparable x with
      | Except.error e => Except.error e
      | Except.ok none => findForwardIndex needle xs (idx + 1)
      | Except.ok (some c) =>
          if comparableEq needle c then
            Except.ok (some idx)
          else
            findForwardIndex needle xs (idx + 1)

def findLastIndexAux
    (needle : XmatchComparable) (hay : List CoercionInput)
    (idx : Nat) (last : Option Nat) : Except XmatchError (Option Nat) :=
  match hay with
  | [] => Except.ok last
  | x :: xs =>
      match toLookupCandidateComparable x with
      | Except.error e => Except.error e
      | Except.ok none => findLastIndexAux needle xs (idx + 1) last
      | Except.ok (some c) =>
          let last' := if comparableEq needle c then some idx else last
          findLastIndexAux needle xs (idx + 1) last'

def findReverseIndex
    (needle : XmatchComparable) (hay : List CoercionInput) : Except XmatchError (Option Nat) :=
  findLastIndexAux needle hay 1 none

def evalXmatchAdapter
    (lookupValue : CoercionInput)
    (lookupArray : List CoercionInput)
    (matchModeArg : Option CoercionInput)
    (searchModeArg : Option CoercionInput) :
    Except XmatchError Value := do
  if lookupArray.isEmpty then
    throw .emptyLookupArray

  let matchMode ← parseOptionalMatchMode matchModeArg
  let searchMode ← parseOptionalSearchMode searchModeArg

  if matchMode ≠ .exact then
    throw (.unsupportedMatchModeForSeed matchMode)

  let needle ← toLookupValueComparable lookupValue
  let foundIdx? : Except XmatchError (Option Nat) :=
    match searchMode with
    | .firstToLast => findForwardIndex needle lookupArray 1
    | .lastToFirst => findReverseIndex needle lookupArray
    | .binaryAscending => Except.error (.unsupportedSearchModeForSeed .binaryAscending)
    | .binaryDescending => Except.error (.unsupportedSearchModeForSeed .binaryDescending)

  let foundIdx ← foundIdx?
  match foundIdx with
  | some idx => Except.ok (Value.number idx)
  | none => Except.error .notAvailable

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
    findReverseIndex (.number 2) [.number 2, .number 1, .number 2] = Except.ok (some 3) := by
  simp [findReverseIndex, findLastIndexAux, comparableEq, toLookupCandidateComparable]

theorem evalXmatch_not_available_lane :
    findForwardIndex (.number 9) [.number 1, .number 2] 1 = Except.ok none := by
  simp [findForwardIndex, comparableEq, toLookupCandidateComparable]

theorem evalXmatch_lookup_array_error_skipped :
    findForwardIndex (.number 2) [.number 1, .error .div0, .number 2] 1 = Except.ok (some 3) := by
  simp [findForwardIndex, comparableEq, toLookupCandidateComparable]

theorem evalXmatch_deterministic
    (lookupValue : CoercionInput)
    (lookupArray : List CoercionInput)
    (matchModeArg : Option CoercionInput)
    (searchModeArg : Option CoercionInput) :
    evalXmatchAdapter lookupValue lookupArray matchModeArg searchModeArg =
      evalXmatchAdapter lookupValue lookupArray matchModeArg searchModeArg := rfl

end OxFunc.Functions
