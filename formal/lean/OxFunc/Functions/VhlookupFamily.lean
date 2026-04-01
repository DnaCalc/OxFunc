import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

inductive VhlookupScalar where
  | number (n : Rat)
  | text (s : String)
  | logical (b : Bool)
  deriving DecidableEq, Repr

inductive LookupMatchMode where
  | exact
  | approximate
  deriving DecidableEq, Repr

def vhlookupBaseMeta : FunctionMeta := {
  functionId := "FUNC.VHLOOKUP_BASE"
  arity := { min := 3, max := 4 }
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

def vlookupMeta : FunctionMeta := { vhlookupBaseMeta with functionId := "FUNC.VLOOKUP" }
def hlookupMeta : FunctionMeta := { vhlookupBaseMeta with functionId := "FUNC.HLOOKUP" }

def exactMatchIndexAux (target : VhlookupScalar) : List VhlookupScalar → Nat → Option Nat
  | [], _ => none
  | x :: xs, idx => if x = target then some idx else exactMatchIndexAux target xs (idx + 1)

def exactMatchIndex (target : VhlookupScalar) (haystack : List VhlookupScalar) : Option Nat :=
  exactMatchIndexAux target haystack 1

def approximateNumberIndexAux (target : Rat) :
    List VhlookupScalar → Nat → Option Nat → Option Nat
  | [], _, best => best
  | VhlookupScalar.number n :: xs, idx, best =>
      let nextBest := if n ≤ target then some idx else best
      approximateNumberIndexAux target xs (idx + 1) nextBest
  | _ :: xs, idx, best => approximateNumberIndexAux target xs (idx + 1) best

def approximateNumberIndex (target : Rat) (haystack : List VhlookupScalar) : Option Nat :=
  approximateNumberIndexAux target haystack 1 none

def lookupIndex (target : VhlookupScalar) (mode : LookupMatchMode) (haystack : List VhlookupScalar) :
    Option Nat :=
  match mode, target with
  | .exact, _ => exactMatchIndex target haystack
  | .approximate, .number n => approximateNumberIndex n haystack
  | .approximate, _ => none

theorem vhlookupFamily_ids_and_arity :
    vlookupMeta.functionId = "FUNC.VLOOKUP"
    ∧ hlookupMeta.functionId = "FUNC.HLOOKUP"
    ∧ vlookupMeta.arity = { min := 3, max := 4 }
    ∧ hlookupMeta.arity = { min := 3, max := 4 } := by
  simp [vhlookupBaseMeta, vlookupMeta, hlookupMeta]

theorem vhlookupFamily_profiles :
    vlookupMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ hlookupMeta.hostInteraction = HostInteractionClass.workbookState
    ∧ vlookupMeta.kernelSignatureClass = KernelSignatureClass.lookupMatch
    ∧ hlookupMeta.kernelSignatureClass = KernelSignatureClass.lookupMatch := by
  simp [vhlookupBaseMeta, vlookupMeta, hlookupMeta]

theorem vhlookup_exact_finds_second_numeric_entry :
    lookupIndex (.number 2) .exact
      [.number 1, .number 2, .number 3] = some 2 := by
  rfl

theorem vhlookup_approximate_below_first_key_is_none :
    lookupIndex (.number (1 / 2)) .approximate
      [.number 1, .number 2, .number 3] = none := by
  native_decide

theorem vhlookup_approximate_returns_last_not_greater_match :
    lookupIndex (.number (29 / 10)) .approximate
      [.number 1, .number 2, .number 3] = some 2 := by
  native_decide

end OxFunc.Functions
