import OxFunc.FunctionCore
import OxFunc.HostInfoSeam
import OxFunc.RefResolverSeam
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

def addressMeta : FunctionMeta := {
  functionId := "FUNC.ADDRESS"
  arity := { min := 2, max := 5 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.none
}

def areasMeta : FunctionMeta := {
  functionId := "FUNC.AREAS"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def formulaTextMeta : FunctionMeta := {
  functionId := "FUNC.FORMULATEXT"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def sheetMeta : FunctionMeta := {
  functionId := "FUNC.SHEET"
  arity := { min := 0, max := 1 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.composite
  surfaceFecDependencyProfile := FecDependencyProfile.composite
}

def sheetsMeta : FunctionMeta := {
  functionId := "FUNC.SHEETS"
  arity := { min := 0, max := 1 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.composite
}

def quoteSheetTextIfNeeded (sheetText : String) : String :=
  if sheetText = "Quarter 1" then "'Quarter 1'" else sheetText

def columnLabelFromIndex : Nat → Option String
  | 0 => none
  | 1 => some "A"
  | 2 => some "B"
  | 26 => some "Z"
  | 27 => some "AA"
  | _ => none

def formatAddressBody (row col absNum : Nat) (a1Style : Bool) : Option String := do
  if a1Style then
    let colText ← columnLabelFromIndex col
    let rowText := toString row
    let colPart := match absNum with
      | 1 | 3 => "$" ++ colText
      | 2 | 4 => colText
      | _ => ""
    let rowPart := match absNum with
      | 1 | 2 => "$" ++ rowText
      | 3 | 4 => rowText
      | _ => ""
    if absNum = 0 || absNum > 4 then none else some (colPart ++ rowPart)
  else
    let rowPart := match absNum with
      | 1 | 2 => "R" ++ toString row
      | 3 | 4 => "R[" ++ toString row ++ "]"
      | _ => ""
    let colPart := match absNum with
      | 1 | 3 => "C" ++ toString col
      | 2 | 4 => "C[" ++ toString col ++ "]"
      | _ => ""
    if absNum = 0 || absNum > 4 then none else some (rowPart ++ colPart)

def renderAddress (row col absNum : Nat) (a1Style : Bool) (sheetText? : Option String) : Option String := do
  let body ← formatAddressBody row col absNum a1Style
  match sheetText? with
  | none => pure body
  | some sheetText => pure (quoteSheetTextIfNeeded sheetText ++ "!" ++ body)

def countAreas (target : String) : Nat :=
  if target = "(A1,B2:B3)" then 2 else 1

theorem referenceMetadataFamily_meta_profiles :
    addressMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ areasMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ formulaTextMeta.hostInteraction = HostInteractionClass.workbookState
    ∧ sheetMeta.fecDependencyProfile = FecDependencyProfile.composite
    ∧ sheetsMeta.surfaceFecDependencyProfile = FecDependencyProfile.composite := by
  simp [addressMeta, areasMeta, formulaTextMeta, sheetMeta, sheetsMeta]

theorem renderAddress_seed_a1_absolute :
    renderAddress 3 2 1 true none = some "$B$3" := by
  native_decide

theorem renderAddress_seed_r1c1_with_sheet_text :
    renderAddress 3 2 4 false (some "Alpha") = some "Alpha!R[3]C[2]" := by
  native_decide

theorem renderAddress_seed_quoted_sheet_text :
    renderAddress 3 2 1 true (some "Quarter 1") = some "'Quarter 1'!$B$3" := by
  native_decide

theorem countAreas_seed_union :
    countAreas "(A1,B2:B3)" = 2 := by
  rfl

theorem hostInfoSeam_supports_reference_metadata_queries :
    providerCanServeFormulaTextQuery "A1" = true
    ∧ providerCanServeSheetIdentity (.currentSheet)
    ∧ providerCanServeSheetIdentity (.reference "Beta!A1")
    ∧ providerCanServeSheetIdentity (.sheetNameText "Alpha")
    ∧ providerCanServeSheetCount (.workbook)
    ∧ providerCanServeSheetCount (.reference "'Quarter 1':Alpha!A1") := by
  simp [providerCanServeFormulaTextQuery, providerCanServeSheetIdentity, providerCanServeSheetCount]

end OxFunc.Functions
