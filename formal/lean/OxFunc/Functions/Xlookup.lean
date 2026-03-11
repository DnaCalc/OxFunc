import OxFunc.FunctionCore
import OxFunc.CoercionPrimitives
import OxFunc.RefResolverSeam
import OxFunc.Functions.Xmatch

namespace OxFunc.Functions

open OxFunc

def xlookupMeta : FunctionMeta := {
  functionId := "FUNC.XLOOKUP"
  arity := { min := 3, max := 6 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.lookupMatch
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

inductive XlookupReturn where
  | value (v : CoercionInput)
  | reference (r : ReferenceToken)
  deriving DecidableEq, Repr

instance : Inhabited XlookupReturn where
  default := .value (.number 0)

def parseXlookupSearchMode : Option CoercionInput → Except CoercionError Bool
  | none => Except.ok true
  | some v =>
      match coerceToNumber v with
      | Except.ok n =>
          if n = 1 then Except.ok true
          else if n = -1 then Except.ok false
          else Except.error (.unsupportedKind "invalid_search_mode")
      | Except.error e => Except.error e

def parseXlookupMatchModeExactOnly : Option CoercionInput → Except CoercionError Unit
  | none => Except.ok ()
  | some v =>
      match coerceToNumber v with
      | Except.ok n =>
          if n = 0 then Except.ok ()
          else Except.error (.unsupportedKind "unsupported_match_mode_seed")
      | Except.error e => Except.error e

def materializeReturn : XlookupReturn → XlookupReturn
  | .value .emptyCell => .value (.number 0)
  | other => other

def selectReturnAt : List XlookupReturn → Nat → Except String XlookupReturn
  | [], _ => Except.error "length_mismatch"
  | x :: _, 0 => Except.ok (materializeReturn x)
  | _ :: xs, idx + 1 => selectReturnAt xs idx

def findXlookupForward
    (needle : XmatchComparable)
    (lookup : List CoercionInput)
    (ret : List XlookupReturn)
    (idx : Nat := 1) : Except String (Option XlookupReturn) :=
  match lookup, ret with
  | [], [] => Except.ok none
  | k :: ks, r :: rs =>
      match toLookupCandidate k with
      | Except.error _ => findXlookupForward needle ks rs (idx + 1)
      | Except.ok (.comparable c) =>
          if comparableEq c needle then
            Except.ok (some (materializeReturn r))
          else
            findXlookupForward needle ks rs (idx + 1)
      | Except.ok .blankCell | Except.ok .skip =>
          findXlookupForward needle ks rs (idx + 1)
  | _, _ => Except.error "length_mismatch"

def evalXlookupSeed
    (lookupValue : CoercionInput)
    (lookupArray : List CoercionInput)
    (returnArray : List XlookupReturn)
    (ifNotFound : Option XlookupReturn := none)
    (matchMode : Option CoercionInput := none)
    (searchMode : Option CoercionInput := none) : Except String XlookupReturn := do
  if lookupArray.length ≠ returnArray.length then
    Except.error "length_mismatch"
  else
    match evalXmatchPosition lookupValue lookupArray matchMode searchMode with
    | Except.ok idx =>
        selectReturnAt returnArray (idx - 1)
    | Except.error .notAvailable =>
        match ifNotFound with
        | some fallback => Except.ok (materializeReturn fallback)
        | none => Except.error "na"
    | Except.error (.invalidMatchMode _) => Except.error "invalid_match_mode"
    | Except.error (.invalidSearchMode _) => Except.error "invalid_search_mode"
    | Except.error (.coercion _) => Except.error "coercion"
    | Except.error .emptyLookupArray => Except.error "na"
    | Except.error (.unsupportedMatchModeForSeed _) => Except.error "unsupported_match_mode_seed"
    | Except.error (.unsupportedSearchModeForSeed _) => Except.error "invalid_search_mode"

theorem parseXlookupMatchModeExactOnly_rejects_seed_non_exact :
    parseXlookupMatchModeExactOnly (some (.number 1)) =
      Except.error (CoercionError.unsupportedKind "unsupported_match_mode_seed") := by
  native_decide

theorem evalXlookupSeed_deterministic
    (lookupValue : CoercionInput)
    (lookupArray : List CoercionInput)
    (returnArray : List XlookupReturn)
    (ifNotFound : Option XlookupReturn)
    (matchMode searchMode : Option CoercionInput) :
    evalXlookupSeed lookupValue lookupArray returnArray ifNotFound matchMode searchMode =
      evalXlookupSeed lookupValue lookupArray returnArray ifNotFound matchMode searchMode := rfl

theorem findXlookupForward_reference_return_preserved :
    findXlookupForward
      (.number 2)
      [.number 1, .number 2, .number 3]
      [.value (.number 10), .reference { kind := .a1, target := "B1" }, .value (.number 30)] =
      Except.ok (some (.reference { kind := .a1, target := "B1" })) := by
  native_decide

theorem xlookupMeta_profiles :
    xlookupMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ xlookupMeta.fecDependencyProfile = FecDependencyProfile.refOnly
    ∧ xlookupMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [xlookupMeta]

theorem evalXlookupSeed_wildcard_reverse :
    evalXlookupSeed (.text "a*") [.text "abc", .text "ade"]
      [.value (.text "first"), .value (.text "last")]
      none (some (.number 2)) (some (.number (-1))) =
      Except.ok (.value (.text "last")) := by
  native_decide

theorem evalXlookupSeed_blank_lookup_matches_blank_cell :
    evalXlookupSeed .emptyCell [.emptyCell, .number 1]
      [.value (.number 10), .value (.number 20)] =
      Except.ok (.value (.number 10)) := by
  native_decide

theorem evalXlookupSeed_blank_return_materializes_zero :
    evalXlookupSeed (.number 1) [.number 1, .number 2]
      [.value .emptyCell, .value (.number 20)] =
      Except.ok (.value (.number 0)) := by
  native_decide

end OxFunc.Functions
