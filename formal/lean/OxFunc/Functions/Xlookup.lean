import OxFunc.FunctionCore
import OxFunc.CoercionPrimitives
import OxFunc.RefResolverSeam

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

inductive XlookupComparable where
  | number (n : Rat)
  | text (s : String)
  | logical (b : Bool)
  deriving DecidableEq, Repr

inductive XlookupReturn where
  | value (v : CoercionInput)
  | reference (r : ReferenceToken)
  deriving DecidableEq, Repr

def toXlookupComparable : CoercionInput → Except CoercionError XlookupComparable
  | .number n => Except.ok (.number n)
  | .text s => Except.ok (.text s)
  | .logical b => Except.ok (.logical b)
  | .error code => Except.error (.worksheetError code)
  | .missingArg => Except.error .missingArg
  | .emptyCell => Except.error .emptyCell

def xlookupCandidateComparable : CoercionInput → Option XlookupComparable
  | .number n => some (.number n)
  | .text s => some (.text s)
  | .logical b => some (.logical b)
  | .error _ => none
  | .missingArg => none
  | .emptyCell => none

def parseXlookupSearchMode : Option CoercionInput → Except CoercionError Bool
  | none => Except.ok true
  | some v =>
      match coerceToNumber v with
      | Except.ok n =>
          if n = 1 then Except.ok true
          else if n = -1 then Except.ok false
          else Except.error (CoercionError.unsupportedKind "invalid_search_mode")
      | Except.error e => Except.error e

def parseXlookupMatchModeExactOnly : Option CoercionInput → Except CoercionError Unit
  | none => Except.ok ()
  | some v =>
      match coerceToNumber v with
      | Except.ok n =>
          if n = 0 then Except.ok ()
          else Except.error (CoercionError.unsupportedKind "unsupported_match_mode_seed")
      | Except.error e => Except.error e

def findXlookupForward
    (needle : XlookupComparable)
    (lookup : List CoercionInput)
    (ret : List XlookupReturn)
    (idx : Nat := 1) : Except String (Option XlookupReturn) :=
      match lookup, ret with
  | [], [] => Except.ok none
  | k :: ks, r :: rs =>
      match xlookupCandidateComparable k with
      | none => findXlookupForward needle ks rs (idx + 1)
      | some c =>
          if c = needle then
            Except.ok (some r)
          else
            findXlookupForward needle ks rs (idx + 1)
  | _, _ => Except.error "length_mismatch"

def evalXlookupSeed
    (lookupValue : CoercionInput)
    (lookupArray : List CoercionInput)
    (returnArray : List XlookupReturn)
  (ifNotFound : Option XlookupReturn := none)
  (matchMode : Option CoercionInput := none)
  (searchMode : Option CoercionInput := none) : Except String XlookupReturn := do
  let _ ← match parseXlookupMatchModeExactOnly matchMode with
    | Except.ok u => Except.ok u
    | Except.error _ => Except.error "unsupported_match_mode_seed"
  let forward ← match parseXlookupSearchMode searchMode with
    | Except.ok b => Except.ok b
    | Except.error _ => Except.error "invalid_search_mode"
  let needle ← match toXlookupComparable lookupValue with
    | Except.ok v => Except.ok v
    | Except.error _ => Except.error "coercion"
  let lookup := if forward then lookupArray else lookupArray.reverse
  let ret := if forward then returnArray else returnArray.reverse
  match findXlookupForward needle lookup ret with
  | Except.error e => Except.error e
  | Except.ok (some v) => Except.ok v
  | Except.ok none =>
      match ifNotFound with
      | some v => Except.ok v
      | none => Except.error "na"

theorem parseXlookupMatchModeExactOnly_rejects_seed_non_exact :
    parseXlookupMatchModeExactOnly (some (.number 1)) =
      Except.error (CoercionError.unsupportedKind "unsupported_match_mode_seed") := by
  simp [parseXlookupMatchModeExactOnly, coerceToNumber]

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
  rfl

theorem xlookupMeta_profiles :
    xlookupMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ xlookupMeta.fecDependencyProfile = FecDependencyProfile.refOnly
    ∧ xlookupMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [xlookupMeta]

end OxFunc.Functions
