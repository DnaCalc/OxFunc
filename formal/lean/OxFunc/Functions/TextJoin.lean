import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

def textJoinMeta : FunctionMeta := {
  functionId := "FUNC.TEXTJOIN"
  arity := { min := 3, max := 255 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.textToText
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def asciiText (s : String) : ExcelText :=
  { utf16CodeUnits := s.toList.map (fun c => c.val.toUInt16) }

def joinUtf16WithDelimiter (delimiter : List Utf16CodeUnit) :
    List (List Utf16CodeUnit) → List Utf16CodeUnit
  | [] => []
  | part :: rest =>
      rest.foldl
        (fun acc next => acc ++ delimiter ++ next)
        part

def textJoinCore (delimiter : ExcelText) (ignoreEmpty : Bool) (parts : List ExcelText) :
    Except WorksheetErrorCode ExcelText :=
  let kept :=
    if ignoreEmpty then
      parts.filter (fun part => !part.utf16CodeUnits.isEmpty)
    else
      parts
  let joined := joinUtf16WithDelimiter delimiter.utf16CodeUnits (kept.map ExcelText.utf16CodeUnits)
  if joined.length <= excelTextMaxUtf16CodeUnits then
    Except.ok { utf16CodeUnits := joined }
  else
    Except.error WorksheetErrorCode.calc

theorem textJoinMeta_profiles :
    textJoinMeta.kernelSignatureClass = KernelSignatureClass.textToText
    ∧ textJoinMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [textJoinMeta]

theorem textJoinCore_basic_join :
    textJoinCore (asciiText ",") true [asciiText "1", asciiText "", asciiText "2"] =
      Except.ok (asciiText "1,2") := by
  rfl

theorem textJoinCore_row_major_flatten_seed :
    textJoinCore (asciiText ",") false
      [asciiText "1", asciiText "2", asciiText "3", asciiText "4"] =
      Except.ok (asciiText "1,2,3,4") := by
  rfl

theorem textJoinCore_numeric_and_logical_delimiter_seed :
    textJoinCore (asciiText "TRUE") false [asciiText "1", asciiText "2"] =
      Except.ok (asciiText "1TRUE2") := by
  rfl

end OxFunc.Functions
